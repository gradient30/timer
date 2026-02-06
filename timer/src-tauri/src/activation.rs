use crate::config::ConfigManager;
use chrono::Local;
use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

const CODE_VERSION: u8 = 1;
const PAYLOAD_LEN: usize = 5;
const SIGNATURE_LEN: usize = 5;
const TOTAL_LEN: usize = PAYLOAD_LEN + SIGNATURE_LEN;

const ACTIVATION_SECRET: [u8; 32] = [
    0x4a, 0x62, 0x1f, 0x8b, 0x77, 0x13, 0x9d, 0x2c,
    0x55, 0xa1, 0x3e, 0x9a, 0x0d, 0x6c, 0x4f, 0x88,
    0x91, 0x2d, 0x6a, 0x70, 0x2f, 0x5c, 0x19, 0x3b,
    0x5d, 0x4e, 0x8f, 0x7c, 0x3a, 0x10, 0x2b, 0x6e,
];
const GENERATOR_PASSWORD: &str = "imdepndc";

#[derive(Debug, Serialize)]
pub struct ActivationStatus {
    pub activated: bool,
}

#[derive(Debug, Serialize)]
pub struct ActivationResult {
    pub activated: bool,
    pub message: String,
}


fn normalize_code(code: &str) -> String {
    code.replace('-', "").trim().to_uppercase()
}

fn format_code(raw: &str) -> String {
    raw.chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

fn compute_signature(payload: &[u8; PAYLOAD_LEN]) -> [u8; SIGNATURE_LEN] {
    let mut mac = HmacSha256::new_from_slice(&ACTIVATION_SECRET)
        .expect("HMAC key length is valid");
    mac.update(payload);
    let result = mac.finalize().into_bytes();
    let mut signature = [0u8; SIGNATURE_LEN];
    signature.copy_from_slice(&result[..SIGNATURE_LEN]);
    signature
}

fn decode_code(code: &str) -> Result<[u8; TOTAL_LEN], String> {
    let normalized = normalize_code(code);
    if normalized.len() != 16 {
        return Err("激活码长度应为16位".to_string());
    }
    let bytes = BASE32_NOPAD
        .decode(normalized.as_bytes())
        .map_err(|_| "激活码格式不正确".to_string())?;
    if bytes.len() != TOTAL_LEN {
        return Err("激活码格式不正确".to_string());
    }
    let mut out = [0u8; TOTAL_LEN];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn verify_code(code: &str) -> Result<(), String> {
    let bytes = decode_code(code)?;
    let mut payload = [0u8; PAYLOAD_LEN];
    payload.copy_from_slice(&bytes[..PAYLOAD_LEN]);
    let mut signature = [0u8; SIGNATURE_LEN];
    signature.copy_from_slice(&bytes[PAYLOAD_LEN..]);

    if payload[0] != CODE_VERSION {
        return Err("激活码版本不匹配".to_string());
    }

    let expected = compute_signature(&payload);
    if expected != signature {
        return Err("激活码无效".to_string());
    }
    Ok(())
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalize_code(code).as_bytes());
    hex::encode(hasher.finalize())
}

pub fn ensure_activated(config_manager: &ConfigManager) -> Result<(), String> {
    let config = config_manager.get()?;
    if config.activation.activated {
        Ok(())
    } else {
        Err("软件未激活，无法使用此功能".to_string())
    }
}

#[tauri::command]
pub fn get_activation_status(
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<ActivationStatus, String> {
    let config = config_manager.get()?;
    Ok(ActivationStatus {
        activated: config.activation.activated,
    })
}

#[tauri::command]
pub fn activate_with_code(
    config_manager: tauri::State<Arc<ConfigManager>>,
    code: String,
) -> Result<ActivationResult, String> {
    let config = config_manager.get()?;
    if config.activation.activated {
        return Err("软件已激活".to_string());
    }

    let normalized = normalize_code(&code);
    if normalized.is_empty() {
        return Err("请输入激活码".to_string());
    }

    verify_code(&normalized)?;

    let now = Local::now().to_rfc3339();
    let hash = hash_code(&normalized);
    config_manager.update(|c| {
        c.activation.activated = true;
        c.activation.activation_code_hash = Some(hash);
        c.activation.activated_at = Some(now);
    })?;

    Ok(ActivationResult {
        activated: true,
        message: "激活成功".to_string(),
    })
}

pub fn generate_activation_code() -> String {
    let mut payload = [0u8; PAYLOAD_LEN];
    payload[0] = CODE_VERSION;
    OsRng.fill_bytes(&mut payload[1..]);

    let signature = compute_signature(&payload);
    let mut bytes = [0u8; TOTAL_LEN];
    bytes[..PAYLOAD_LEN].copy_from_slice(&payload);
    bytes[PAYLOAD_LEN..].copy_from_slice(&signature);

    let raw = BASE32_NOPAD.encode(&bytes);
    format_code(&raw)
}

#[tauri::command]
pub fn generate_activation_codes(password: String) -> Result<Vec<String>, String> {
    if password != GENERATOR_PASSWORD {
        return Err("口令错误".to_string());
    }

    let mut codes = Vec::with_capacity(5);
    for _ in 0..5 {
        codes.push(generate_activation_code());
    }
    Ok(codes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify() {
        let code = generate_activation_code();
        assert!(verify_code(&code).is_ok());
    }
}
