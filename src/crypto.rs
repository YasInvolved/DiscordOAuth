use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};

use rand::{Rng, rngs::{StdRng}};

pub fn encrypt_token(plain_text: &str, key_bytes: &[u8; 32]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let cipher = Aes256Gcm::new_from_slice(key_bytes)?;

    let mut rng: StdRng = rand::make_rng();
    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::try_from(nonce_bytes)?;

    let ciphertext = cipher.encrypt(&nonce, plain_text.as_bytes())
        .map_err(|e| format!("Encryption failure: {e}"))?;

    let mut payload = nonce_bytes.to_vec();
    payload.extend(ciphertext);
    Ok(payload)
}

pub fn decrypt_token(payload: &[u8], key_bytes: &[u8; 32]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if payload.len() < 12 {
        return Err("Invalid payload length".into());
    }

    let cipher = Aes256Gcm::new_from_slice(key_bytes)?;
    let (nonce_bytes, ciphertext) = payload.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes)?;

    let decrypted_bytes = cipher.decrypt(&nonce, ciphertext)
        .map_err(|e| format!("Decryption failure: {e}"))?;

    Ok(String::from_utf8(decrypted_bytes)?)
}