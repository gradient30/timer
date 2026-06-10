use crate::config::ConfigManager;
use chrono::Local;
use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};

type HmacSha256 = Hmac<Sha256>;

const CODE_VERSION: u8 = 1;
const PAYLOAD_LEN: usize = 5;
const SIGNATURE_LEN: usize = 5;
const TOTAL_LEN: usize = PAYLOAD_LEN + SIGNATURE_LEN;

static ACTIVATION_SECRET: OnceLock<[u8; 32]> = OnceLock::new();

#[derive(Debug, Serialize)]
pub struct ActivationStatus {
    pub activated: bool,
    #[serde(rename = "admin_enabled")]
    pub admin_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ActivationResult {
    pub activated: bool,
    pub message: String,
}

fn activation_secret_bytes() -> [u8; 32] {
    *ACTIVATION_SECRET.get_or_init(|| {
        let secret_hex = env!("TIMER_ACTIVATION_SECRET_HEX");
        let bytes = hex::decode(secret_hex).expect("invalid TIMER_ACTIVATION_SECRET_HEX");
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        out
    })
}

#[cfg(feature = "activation-admin")]
fn generator_password() -> &'static str {
    env!("TIMER_GENERATOR_PASSWORD")
}

pub fn is_activation_admin_enabled() -> bool {
    cfg!(feature = "activation-admin")
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
    let mut mac = HmacSha256::new_from_slice(&activation_secret_bytes())
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
        admin_enabled: is_activation_admin_enabled(),
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

#[cfg(feature = "activation-admin")]
#[tauri::command]
pub fn generate_activation_codes(password: String) -> Result<Vec<String>, String> {
    if password != generator_password() {
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
