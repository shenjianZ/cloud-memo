use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;

use crate::models::error::{AppError, Result};

/// 加密工具类
pub struct CryptoService;

impl CryptoService {
    /// 应用特定盐值（硬编码，防止跨应用密钥重用）
    const APP_SALT: &'static [u8] = b"markdown-notes-app-salt-abcdefg-2026";

    /// PBKDF2 迭代次数（OWASP 推荐的最小迭代次数）
    const ITERATIONS: u32 = 100_000;

    /// 从 device_id 派生加密密钥
    ///
    /// 使用 PBKDF2-HMAC-SHA256 算法从 device_id 派生一个 32 字节的加密密钥
    pub fn derive_key_from_device_id(device_id: &str) -> [u8; 32] {
        let mut key = [0u8; 32];

        pbkdf2_hmac::<Sha256>(
            device_id.as_bytes(),
            Self::APP_SALT,
            Self::ITERATIONS,
            &mut key,
        );

        key
    }

    /// 加密 token
    ///
    /// 使用 Aes256Gcm 加密算法加密 token，返回 base64 编码的字符串
    /// 格式: base64(nonce + ciphertext)
    pub fn encrypt_token(token: &str, key: &[u8; 32]) -> Result<String> {
        let cipher = Aes256Gcm::new(key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, token.as_bytes())
            .map_err(|e| AppError::EncryptionError(format!("加密失败: {}", e)))?;

        // 将 nonce 和 ciphertext 合并并编码为 base64
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&result))
    }

    /// 解密 token
    ///
    /// 使用 Aes256Gcm 解密算法解密 token
    pub fn decrypt_token(encrypted: &str, key: &[u8; 32]) -> Result<String> {
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| AppError::EncryptionError(format!("解码失败: {}", e)))?;

        if data.len() < 12 {
            return Err(AppError::EncryptionError(
                "无效的加密数据".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new(key.into());
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::EncryptionError(format!("解密失败: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::EncryptionError(format!("无效的 UTF-8: {}", e)))
    }

    /// 加密字符串并保存到文件
    ///
    /// 用于保存敏感数据（如 refresh token）到本地文件
    pub fn encrypt_to_file(data: &str, file_path: &PathBuf, key: &[u8; 32]) -> Result<()> {
        let encrypted = Self::encrypt_token(data, key)?;

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::EncryptionError(format!("创建目录失败: {}", e))
            })?;
        }

        fs::write(file_path, encrypted)
            .map_err(|e| AppError::EncryptionError(format!("写入文件失败: {}", e)))?;

        Ok(())
    }

    /// 从文件读取并解密数据
    ///
    /// 用于从本地文件读取敏感数据
    pub fn decrypt_from_file(file_path: &PathBuf, key: &[u8; 32]) -> Result<String> {
        let encrypted = fs::read_to_string(file_path)
            .map_err(|e| AppError::EncryptionError(format!("读取文件失败: {}", e)))?;

        Self::decrypt_token(&encrypted, key)
    }

    /// 生成随机设备 ID
    pub fn generate_device_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_token() {
        let device_id = "test-device-123";
        let key = CryptoService::derive_key_from_device_id(device_id);
        let original_token = "my-secret-access-token";

        // 加密
        let encrypted =
            CryptoService::encrypt_token(original_token, &key).expect("Encryption failed");

        println!("Encrypted: {}", encrypted);

        // 解密
        let decrypted = CryptoService::decrypt_token(&encrypted, &key).expect("Decryption failed");

        assert_eq!(original_token, decrypted);
        println!("✅ Encryption/Decryption test passed");
    }

    #[test]
    fn test_key_derivation_consistency() {
        let device_id = "consistent-device-id";

        let key1 = CryptoService::derive_key_from_device_id(device_id);
        let key2 = CryptoService::derive_key_from_device_id(device_id);

        assert_eq!(key1, key2, "Key derivation should be deterministic");
        println!("✅ Key derivation consistency test passed");
    }

    #[test]
    fn test_key_derivation_uniqueness() {
        let device_id1 = "device-1";
        let device_id2 = "device-2";

        let key1 = CryptoService::derive_key_from_device_id(device_id1);
        let key2 = CryptoService::derive_key_from_device_id(device_id2);

        assert_ne!(
            key1, key2,
            "Different device IDs should produce different keys"
        );
        println!("✅ Key derivation uniqueness test passed");
    }

    #[test]
    fn test_wrong_key_fails() {
        let device_id1 = "device-1";
        let device_id2 = "device-2";

        let key1 = CryptoService::derive_key_from_device_id(device_id1);
        let key2 = CryptoService::derive_key_from_device_id(device_id2);

        let token = "my-secret-token";
        let encrypted = CryptoService::encrypt_token(token, &key1).expect("Encryption failed");

        // 使用错误的密钥解密应该失败
        let result = CryptoService::decrypt_token(&encrypted, &key2);

        assert!(result.is_err(), "Decryption with wrong key should fail");
        println!("✅ Wrong key failure test passed");
    }
}
