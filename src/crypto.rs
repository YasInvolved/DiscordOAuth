use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::{TryRng, rngs::SysRng};
use std::env;

fn get_cipher() -> Aes256Gcm {
    let key_str = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let key_bytes = hex::decode(key_str).expect("Key must be a valid hex string");
    
    Aes256Gcm::new_from_slice(&key_bytes)
        .expect("Invalid key length: ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars)")
}

pub fn encrypt_token(token: &str) -> String {
    let cipher = get_cipher();

    let mut nonce_bytes = [0u8; 12];
    SysRng.try_fill_bytes(&mut nonce_bytes).expect("Encryption Failure");
    let nonce = Nonce::try_from(nonce_bytes).expect("Encryption Error");

    let ciphertext = cipher.encrypt(&nonce, token.as_bytes())
        .expect("Encrytpion Error");

    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);

    BASE64.encode(combined)
}

pub fn decrypt_token(encrypted_base64: &str) -> String {
    let cipher = get_cipher();

    let combined = BASE64.decode(encrypted_base64)
        .expect("Failed to decode Base64");

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes).expect("Decryption failure - failed to transform nonce bytes to object");

    let plaintext_bytes = cipher.decrypt(&nonce, ciphertext)
        .expect("Decryption failure - invalid key or tampered data");

    String::from_utf8(plaintext_bytes).unwrap()
}