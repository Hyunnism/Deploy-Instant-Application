// storage/secrets.rs
// Enkripsi rahasia PAT & kredensial dengan AES-256-GCM (PRD Bagian 12A.3 & Fase 4)

use crate::error::AppError;
use crate::storage::db::Database;
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use std::fs;
use std::path::Path;

pub const SETTING_KEY_GITHUB_PAT: &str = "github_pat";
const MASTER_KEY_FILENAME: &str = ".master.key";

/// Mengambil atau membuat master key 256-bit (32 byte) pada direktori aplikasi
pub fn get_or_create_master_key(key_dir: &Path) -> Result<Key<Aes256Gcm>, AppError> {
    if !key_dir.exists() {
        fs::create_dir_all(key_dir)
            .map_err(|e| AppError::Internal(format!("Gagal membuat direktori kunci: {e}")))?;
    }

    let key_file = key_dir.join(MASTER_KEY_FILENAME);
    if key_file.exists() {
        let bytes = fs::read(&key_file)
            .map_err(|e| AppError::Internal(format!("Gagal membaca file kunci rahasia: {e}")))?;
        if bytes.len() == 32 {
            return Ok(*Key::<Aes256Gcm>::from_slice(&bytes));
        }
    }

    // Generate fresh 32-byte key
    let key = Aes256Gcm::generate_key(OsRng);
    fs::write(&key_file, key.as_slice())
        .map_err(|e| AppError::Internal(format!("Gagal menyimpan file kunci rahasia: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_file, fs::Permissions::from_mode(0o600));
    }

    Ok(key)
}

/// Enkripsi teks plaintext menggunakan AES-256-GCM
pub fn encrypt_secret(key: &Key<Aes256Gcm>, plaintext: &str) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits unique nonce

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Internal(format!("Gagal enkripsi secret: {e}")))?;

    let nonce_hex = to_hex(nonce.as_slice());
    let ct_hex = to_hex(&ciphertext);

    Ok(format!("{}:{}", nonce_hex, ct_hex))
}

/// Dekripsi ciphertext terenkripsi ke teks plaintext
pub fn decrypt_secret(key: &Key<Aes256Gcm>, payload: &str) -> Result<String, AppError> {
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::Internal(
            "Format secret terenkripsi tidak valid".into(),
        ));
    }

    let nonce_bytes = from_hex(parts[0])
        .map_err(|e| AppError::Internal(format!("Nonce hex tidak valid: {e}")))?;
    let ct_bytes = from_hex(parts[1])
        .map_err(|e| AppError::Internal(format!("Ciphertext hex tidak valid: {e}")))?;

    if nonce_bytes.len() != 12 {
        return Err(AppError::Internal("Panjang nonce tidak valid".into()));
    }

    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher.decrypt(nonce, ct_bytes.as_ref()).map_err(|e| {
        AppError::Internal(format!("Gagal dekripsi secret (kunci salah / rusak): {e}"))
    })?;

    String::from_utf8(plaintext_bytes)
        .map_err(|e| AppError::Internal(format!("Plaintext bukan UTF-8 valid: {e}")))
}

/// Simpan PAT GitHub terenkripsi ke tabel settings
pub fn save_pat(db: &Database, key_dir: &Path, pat: &str) -> Result<(), AppError> {
    let trimmed = pat.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Token GitHub PAT tidak boleh kosong".into(),
        ));
    }

    let key = get_or_create_master_key(key_dir)?;
    let encrypted = encrypt_secret(&key, trimmed)?;

    db.set_setting(SETTING_KEY_GITHUB_PAT, &encrypted)
}

/// Ambil dan dekripsi PAT GitHub dari tabel settings
pub fn get_pat(db: &Database, key_dir: &Path) -> Result<Option<String>, AppError> {
    let stored = db.get_setting(SETTING_KEY_GITHUB_PAT)?;
    let encrypted = match stored {
        Some(s) if !s.trim().is_empty() => s,
        _ => return Ok(None),
    };

    let key = get_or_create_master_key(key_dir)?;
    let plaintext = decrypt_secret(&key, &encrypted)?;
    Ok(Some(plaintext))
}

/// Periksa apakah PAT GitHub sudah diset (tanpa dekripsi / membocorkan token)
pub fn has_pat(db: &Database) -> Result<bool, AppError> {
    let stored = db.get_setting(SETTING_KEY_GITHUB_PAT)?;
    Ok(stored.is_some_and(|s| !s.trim().is_empty()))
}

/// Hapus PAT GitHub yang tersimpan
pub fn delete_pat(db: &Database) -> Result<(), AppError> {
    db.delete_setting(SETTING_KEY_GITHUB_PAT)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("Panjang hex ganjil".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("Karakter hex tidak valid: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("test_sec_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);

        let key = get_or_create_master_key(&temp_dir).expect("Key creation failed");
        let token = "ghp_1234567890abcdefghijklmnopqrstuvwxyz";

        let encrypted = encrypt_secret(&key, token).expect("Encryption failed");
        assert_ne!(encrypted, token);
        assert!(!encrypted.contains(token));

        let decrypted = decrypt_secret(&key, &encrypted).expect("Decryption failed");
        assert_eq!(decrypted, token);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_save_get_delete_pat() {
        let db = Database::open_in_memory().expect("DB open failed");
        let temp_dir = std::env::temp_dir().join(format!("test_pat_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);

        assert!(!has_pat(&db).unwrap());
        assert_eq!(get_pat(&db, &temp_dir).unwrap(), None);

        let pat = "github_pat_11ABCD_mock_secret_token";
        save_pat(&db, &temp_dir, pat).expect("Save PAT failed");

        assert!(has_pat(&db).unwrap());
        assert_eq!(get_pat(&db, &temp_dir).unwrap(), Some(pat.to_string()));

        delete_pat(&db).expect("Delete PAT failed");
        assert!(!has_pat(&db).unwrap());
        assert_eq!(get_pat(&db, &temp_dir).unwrap(), None);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
