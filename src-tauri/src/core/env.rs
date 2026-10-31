// core/env.rs
// Parser file .env mandiri standar POSIX (PRD Bagian 9.3 & 13 Fase 6)
// Mematuhi /ponytail: zero-dependency, standard library only, tangguh.

use crate::error::AppError;
use crate::models::EnvVarItem;

/// Validasi nama key environment variable POSIX standar:
/// Harus diawali dengan huruf (a-z, A-Z) atau garis bawah (_),
/// diikuti huruf, angka, atau garis bawah.
pub fn is_valid_env_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let mut chars = key.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Menghapus tanda kutip ("..." atau '...') dan mengurai escape sequence bila ada
pub fn unquote_env_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        let is_double_quoted = trimmed.starts_with('"');
        let inner = &trimmed[1..trimmed.len() - 1];

        if is_double_quoted {
            // Dalam tanda petik ganda, proses escape sequences (\n, \t, \", \\)
            let mut result = String::with_capacity(inner.len());
            let mut chars = inner.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some(other) => {
                            result.push('\\');
                            result.push(other);
                        }
                        None => result.push('\\'),
                    }
                } else {
                    result.push(c);
                }
            }
            result
        } else {
            // Dalam tanda petik tunggal, simpan verbatim sesuai POSIX shell single-quote
            inner.to_string()
        }
    } else {
        // Tanpa kutip: jika ada komentar di ujung (misal: VAL # komentar), bersihkan
        let val_part = if let Some(idx) = trimmed.find(" #") {
            &trimmed[..idx]
        } else {
            trimmed
        };
        val_part.trim().to_string()
    }
}

/// Mem-parse konten teks berformat .env menjadi daftar EnvVarItem
pub fn parse_dotenv(content: &str) -> Result<Vec<EnvVarItem>, AppError> {
    let mut vars = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Abaikan baris kosong dan baris komentar
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Dukung awalan opsional "export "
        let clean_line = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim();

        if let Some((raw_key, raw_val)) = clean_line.split_once('=') {
            let key = raw_key.trim();
            if !is_valid_env_key(key) {
                continue;
            }
            let value = unquote_env_value(raw_val);
            vars.push(EnvVarItem {
                key: key.to_string(),
                value,
            });
        }
    }

    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_env_key() {
        assert!(is_valid_env_key("PORT"));
        assert!(is_valid_env_key("NODE_ENV"));
        assert!(is_valid_env_key("_SECRET_KEY_123"));
        assert!(!is_valid_env_key(""));
        assert!(!is_valid_env_key("123PORT"));
        assert!(!is_valid_env_key("NODE-ENV"));
        assert!(!is_valid_env_key("KEY WITH SPACE"));
        assert!(!is_valid_env_key("KEY;INJECTION"));
    }

    #[test]
    fn test_unquote_env_value() {
        assert_eq!(unquote_env_value("\"hello world\""), "hello world");
        assert_eq!(unquote_env_value("'single quote'"), "single quote");
        assert_eq!(unquote_env_value("\"line1\\nline2\""), "line1\nline2");
        assert_eq!(unquote_env_value("plain_value"), "plain_value");
        assert_eq!(
            unquote_env_value("postgres://user:pass@host/db?ssl=true"),
            "postgres://user:pass@host/db?ssl=true"
        );
        assert_eq!(unquote_env_value("value # this is comment"), "value");
    }

    #[test]
    fn test_parse_dotenv_full_scenarios() {
        let env_text = r#"
# Komentar awal
PORT=3000
export NODE_ENV="production"

# Database URL dengan tanda '=' ganda
DATABASE_URL=postgresql://user:pass@localhost:5432/mydb?sslmode=disable&foo=bar

# Single quote string
API_SECRET='super-secret-key-without-escapes'

# Multi-line newline escape
GREETING="Hello\nWorld"

# Komentar inline
APP_NAME=DeployInstant # Komentar inline disini
INVALID-KEY=should_be_skipped
"#;

        let parsed = parse_dotenv(env_text).expect("Gagal parse dotenv");
        assert_eq!(parsed.len(), 6);

        assert_eq!(parsed[0].key, "PORT");
        assert_eq!(parsed[0].value, "3000");

        assert_eq!(parsed[1].key, "NODE_ENV");
        assert_eq!(parsed[1].value, "production");

        assert_eq!(parsed[2].key, "DATABASE_URL");
        assert_eq!(
            parsed[2].value,
            "postgresql://user:pass@localhost:5432/mydb?sslmode=disable&foo=bar"
        );

        assert_eq!(parsed[3].key, "API_SECRET");
        assert_eq!(parsed[3].value, "super-secret-key-without-escapes");

        assert_eq!(parsed[4].key, "GREETING");
        assert_eq!(parsed[4].value, "Hello\nWorld");

        assert_eq!(parsed[5].key, "APP_NAME");
        assert_eq!(parsed[5].value, "DeployInstant");
    }
}
