use crate::error::{CoreError, CoreResult};
use crate::storage::crypto::{
    decrypt_bytes, encrypt_bytes, generate_dek_32, EncryptedBlob, EncryptionAlgorithm,
};
use crate::storage::key_management::{get_or_create_kek, KeyStorage};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub vault_id: String,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub encryption_at_rest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultKeyState {
    key_id: String,
    kek_storage: KeyStorage,
    wrapped_dek: EncryptedBlob,
}

pub struct VaultStorage {
    root: PathBuf,
    dek: [u8; 32],
    cfg: VaultConfig,
    key_state: VaultKeyState,
}

impl VaultStorage {
    pub fn create(root: impl AsRef<Path>, cfg: VaultConfig) -> CoreResult<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("sqlite"))?;
        fs::create_dir_all(root.join("blobs"))?;
        fs::create_dir_all(root.join("caches"))?;
        fs::create_dir_all(root.join("meta"))?;
        fs::write(
            root.join("meta").join("vault_config.json"),
            serde_json::to_vec_pretty(&cfg)?,
        )?;

        let dek = generate_dek_32();
        let kek_material =
            get_or_create_kek(&cfg.vault_id, &root.join("meta").join("kek_fallback.bin"))?;
        let wrapped_dek = encrypt_bytes(EncryptionAlgorithm::AES_256_GCM, &kek_material.kek, &dek)?;
        let key_state = VaultKeyState {
            key_id: "kek_v1".to_string(),
            kek_storage: kek_material.storage,
            wrapped_dek,
        };
        fs::write(
            root.join("meta").join("key_state.json"),
            serde_json::to_vec_pretty(&key_state)?,
        )?;
        Ok(Self {
            root,
            dek,
            cfg,
            key_state,
        })
    }

    pub fn open(root: impl AsRef<Path>) -> CoreResult<Self> {
        let root = root.as_ref().to_path_buf();
        let cfg_bytes = fs::read(root.join("meta").join("vault_config.json"))?;
        let cfg: VaultConfig = serde_json::from_slice(&cfg_bytes)?;
        let key_state_bytes = fs::read(root.join("meta").join("key_state.json"))?;
        let key_state: VaultKeyState = serde_json::from_slice(&key_state_bytes)?;
        let kek_material =
            get_or_create_kek(&cfg.vault_id, &root.join("meta").join("kek_fallback.bin"))?;
        let dek_bytes = decrypt_bytes(&key_state.wrapped_dek, &kek_material.kek)?;
        if dek_bytes.len() != 32 {
            return Err(CoreError::InvalidInput(
                "wrapped DEK did not unwrap to 32 bytes".to_string(),
            ));
        }
        let mut dek = [0u8; 32];
        dek.copy_from_slice(&dek_bytes);
        Ok(Self {
            root,
            dek,
            cfg,
            key_state,
        })
    }

    pub fn write_blob(&self, blob_id: &str, plaintext: &[u8]) -> CoreResult<()> {
        let path = self.root.join("blobs").join(format!("{}.bin", blob_id));
        if self.cfg.encryption_at_rest {
            let enc = encrypt_bytes(self.cfg.encryption_algorithm, &self.dek, plaintext)?;
            fs::write(path, serde_json::to_vec(&enc)?)?;
        } else {
            fs::write(path, plaintext)?;
        }
        Ok(())
    }

    pub fn read_blob(&self, blob_id: &str) -> CoreResult<Vec<u8>> {
        let path = self.root.join("blobs").join(format!("{}.bin", blob_id));
        let bytes = fs::read(path)?;
        if self.cfg.encryption_at_rest {
            let enc: EncryptedBlob = serde_json::from_slice(&bytes)?;
            decrypt_bytes(&enc, &self.dek)
        } else {
            Ok(bytes)
        }
    }

    pub fn write_sqlite_bytes(&self, db_bytes: &[u8]) -> CoreResult<()> {
        let path = self.root.join("sqlite").join("vault.db.enc");
        if self.cfg.encryption_at_rest {
            let enc = encrypt_bytes(self.cfg.encryption_algorithm, &self.dek, db_bytes)?;
            fs::write(path, serde_json::to_vec(&enc)?)?;
        } else {
            fs::write(path, db_bytes)?;
        }
        Ok(())
    }

    pub fn read_sqlite_bytes(&self) -> CoreResult<Vec<u8>> {
        let path = self.root.join("sqlite").join("vault.db.enc");
        let bytes = fs::read(path)?;
        if self.cfg.encryption_at_rest {
            let enc: EncryptedBlob = serde_json::from_slice(&bytes)?;
            decrypt_bytes(&enc, &self.dek)
        } else {
            Ok(bytes)
        }
    }

    pub fn rotate_dek(&mut self, new_key_id: &str) -> CoreResult<serde_json::Value> {
        let old_key_id = self.key_state.key_id.clone();
        let old_dek = self.dek;
        let new_dek = generate_dek_32();

        // Re-encrypt sqlite
        if self.root.join("sqlite").join("vault.db.enc").exists() {
            let current_db = self.read_sqlite_bytes()?;
            let enc = encrypt_bytes(self.cfg.encryption_algorithm, &new_dek, &current_db)?;
            fs::write(
                self.root.join("sqlite").join("vault.db.enc"),
                serde_json::to_vec(&enc)?,
            )?;
        }

        // Re-encrypt blobs
        let blobs_dir = self.root.join("blobs");
        if blobs_dir.exists() {
            for ent in fs::read_dir(&blobs_dir)? {
                let ent = ent?;
                let p = ent.path();
                if p.extension().and_then(|x| x.to_str()) != Some("bin") {
                    continue;
                }
                let bytes = fs::read(&p)?;
                let pt = if self.cfg.encryption_at_rest {
                    let enc: EncryptedBlob = serde_json::from_slice(&bytes)?;
                    decrypt_bytes(&enc, &old_dek)?
                } else {
                    bytes
                };
                if self.cfg.encryption_at_rest {
                    let enc = encrypt_bytes(self.cfg.encryption_algorithm, &new_dek, &pt)?;
                    fs::write(&p, serde_json::to_vec(&enc)?)?;
                } else {
                    fs::write(&p, pt)?;
                }
            }
        }

        // Wrap new DEK with KEK from configured storage.
        let kek_material = get_or_create_kek(
            &self.cfg.vault_id,
            &self.root.join("meta").join("kek_fallback.bin"),
        )?;
        let wrapped_dek = encrypt_bytes(
            EncryptionAlgorithm::AES_256_GCM,
            &kek_material.kek,
            &new_dek,
        )?;
        self.key_state = VaultKeyState {
            key_id: new_key_id.to_string(),
            kek_storage: kek_material.storage,
            wrapped_dek,
        };
        fs::write(
            self.root.join("meta").join("key_state.json"),
            serde_json::to_vec_pretty(&self.key_state)?,
        )?;
        self.dek = new_dek;

        Ok(serde_json::json!({
            "old_key_id": old_key_id,
            "new_key_id": new_key_id
        }))
    }

    pub fn encryption_status_audit_details(&self) -> serde_json::Value {
        serde_json::json!({
            "encryption_at_rest": self.cfg.encryption_at_rest,
            "algorithm": match self.cfg.encryption_algorithm {
                EncryptionAlgorithm::XCHACHA20_POLY1305 => "XCHACHA20_POLY1305",
                EncryptionAlgorithm::AES_256_GCM => "AES_256_GCM",
            },
            "key_storage": match self.key_state.kek_storage {
                KeyStorage::MACOS_KEYCHAIN => "MACOS_KEYCHAIN",
                KeyStorage::WINDOWS_DPAPI => "WINDOWS_DPAPI",
                KeyStorage::FILE_FALLBACK => "FILE_FALLBACK",
            }
        })
    }
}
