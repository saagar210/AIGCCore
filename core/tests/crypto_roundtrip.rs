use aigc_core::storage::crypto::{
    decrypt_bytes, encrypt_bytes, generate_dek_32, EncryptionAlgorithm,
};

#[test]
fn xchacha20_poly1305_roundtrip() {
    let key = generate_dek_32();
    let pt = b"secret payload";
    let enc = encrypt_bytes(EncryptionAlgorithm::XCHACHA20_POLY1305, &key, pt).unwrap();
    let dec = decrypt_bytes(&enc, &key).unwrap();
    assert_eq!(dec, pt);
}

#[test]
fn aes_256_gcm_roundtrip() {
    let key = generate_dek_32();
    let pt = b"secret payload";
    let enc = encrypt_bytes(EncryptionAlgorithm::AES_256_GCM, &key, pt).unwrap();
    let dec = decrypt_bytes(&enc, &key).unwrap();
    assert_eq!(dec, pt);
}
