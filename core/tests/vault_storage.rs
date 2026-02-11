use aigc_core::storage::crypto::EncryptionAlgorithm;
use aigc_core::storage::vault::{VaultConfig, VaultStorage};

#[test]
fn vault_blob_and_sqlite_are_encrypted_at_rest() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = VaultConfig {
        vault_id: "v_test".to_string(),
        encryption_algorithm: EncryptionAlgorithm::XCHACHA20_POLY1305,
        encryption_at_rest: true,
    };
    let vault = VaultStorage::create(dir.path(), cfg).unwrap();

    let blob_pt = b"top-secret-blob";
    let db_pt = b"sqlite-raw-bytes";
    vault.write_blob("b1", blob_pt).unwrap();
    vault.write_sqlite_bytes(db_pt).unwrap();

    let blob_ct_disk = std::fs::read(dir.path().join("blobs").join("b1.bin")).unwrap();
    let db_ct_disk = std::fs::read(dir.path().join("sqlite").join("vault.db.enc")).unwrap();
    assert!(!String::from_utf8_lossy(&blob_ct_disk).contains("top-secret-blob"));
    assert!(!String::from_utf8_lossy(&db_ct_disk).contains("sqlite-raw-bytes"));

    let opened = VaultStorage::open(dir.path()).unwrap();
    assert_eq!(opened.read_blob("b1").unwrap(), blob_pt);
    assert_eq!(opened.read_sqlite_bytes().unwrap(), db_pt);
}

#[test]
fn vault_dek_rotation_reencrypts_and_preserves_readability() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = VaultConfig {
        vault_id: "v_rotate".to_string(),
        encryption_algorithm: EncryptionAlgorithm::AES_256_GCM,
        encryption_at_rest: true,
    };
    let mut vault = VaultStorage::create(dir.path(), cfg).unwrap();
    vault.write_blob("b1", b"alpha").unwrap();
    vault.write_sqlite_bytes(b"db-bytes").unwrap();

    let ev = vault.rotate_dek("kek_v2").unwrap();
    assert_eq!(
        ev.get("old_key_id").and_then(|x| x.as_str()).unwrap(),
        "kek_v1"
    );
    assert_eq!(
        ev.get("new_key_id").and_then(|x| x.as_str()).unwrap(),
        "kek_v2"
    );

    assert_eq!(vault.read_blob("b1").unwrap(), b"alpha");
    assert_eq!(vault.read_sqlite_bytes().unwrap(), b"db-bytes");

    let reopened = VaultStorage::open(dir.path()).unwrap();
    assert_eq!(reopened.read_blob("b1").unwrap(), b"alpha");
    assert_eq!(reopened.read_sqlite_bytes().unwrap(), b"db-bytes");
}

#[test]
fn encryption_status_payload_contains_required_fields() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = VaultConfig {
        vault_id: "v_status".to_string(),
        encryption_algorithm: EncryptionAlgorithm::XCHACHA20_POLY1305,
        encryption_at_rest: true,
    };
    let vault = VaultStorage::create(dir.path(), cfg).unwrap();
    let d = vault.encryption_status_audit_details();
    assert_eq!(
        d.get("encryption_at_rest").and_then(|x| x.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("algorithm").and_then(|x| x.as_str()),
        Some("XCHACHA20_POLY1305")
    );
    assert!(d.get("key_storage").is_some());
}
