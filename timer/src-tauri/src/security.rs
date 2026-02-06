use crate::{config::ConfigManager, RuntimeFlags};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use chrono::{DateTime, Duration, Local};
use serde::Serialize;
use std::sync::Arc;

const MAX_FAILED_ATTEMPTS: u32 = 3;
const LOCK_MINUTES: i64 = 5;

#[derive(Debug, Serialize)]
pub struct SecurityStatus {
    pub password_set: bool,
    pub security_question: Option<String>,
    pub lock_remaining_seconds: u64,
    pub remaining_attempts: u32,
    pub max_attempts: u32,
    pub safe_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub ok: bool,
    pub locked: bool,
    pub lock_remaining_seconds: u64,
    pub remaining_attempts: u32,
}

fn normalize_password(password: &str) -> String {
    password.trim().to_string()
}

fn normalize_answer(answer: &str) -> String {
    answer.trim().to_lowercase()
}

fn hash_secret(secret: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(secret.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("密码哈希失败: {}", e))
}

fn verify_secret(hash: &str, secret: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("解析密码哈希失败: {}", e))?;
    Ok(Argon2::default()
        .verify_password(secret.as_bytes(), &parsed_hash)
        .is_ok())
}

fn lock_remaining_seconds(lock_until: &Option<String>) -> u64 {
    if let Some(lock_until) = lock_until {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(lock_until) {
            let now = Local::now();
            let diff = parsed.with_timezone(&Local) - now;
            if diff.num_seconds() > 0 {
                return diff.num_seconds() as u64;
            }
        }
    }
    0
}

fn reset_lock_if_expired(config_manager: &ConfigManager) -> Result<(), String> {
    let config = config_manager.get()?;
    let lock_remaining = lock_remaining_seconds(&config.security.lock_until);
    if lock_remaining == 0 && config.security.lock_until.is_some() {
        config_manager.update(|c| {
            c.security.lock_until = None;
            c.security.failed_attempts = 0;
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_security_status(
    config_manager: tauri::State<Arc<ConfigManager>>,
    runtime_flags: tauri::State<RuntimeFlags>,
) -> Result<SecurityStatus, String> {
    reset_lock_if_expired(&config_manager)?;
    let config = config_manager.get()?;
    let lock_remaining = lock_remaining_seconds(&config.security.lock_until);
    let remaining_attempts = MAX_FAILED_ATTEMPTS.saturating_sub(config.security.failed_attempts);

    Ok(SecurityStatus {
        password_set: config.security.password_hash.is_some(),
        security_question: config.security.security_question.clone(),
        lock_remaining_seconds: lock_remaining,
        remaining_attempts,
        max_attempts: MAX_FAILED_ATTEMPTS,
        safe_mode: runtime_flags.safe_mode,
    })
}

#[tauri::command]
pub fn setup_password(
    config_manager: tauri::State<Arc<ConfigManager>>,
    password: String,
    security_question: String,
    security_answer: String,
) -> Result<(), String> {
    let password = normalize_password(&password);
    if password.len() < 4 {
        return Err("密码长度至少4位".to_string());
    }
    let question = security_question.trim();
    if question.is_empty() {
        return Err("请选择密保问题".to_string());
    }
    let answer = normalize_answer(&security_answer);
    if answer.is_empty() {
        return Err("请输入密保答案".to_string());
    }

    let config = config_manager.get()?;
    if config.security.password_hash.is_some() {
        return Err("密码已设置".to_string());
    }

    let password_hash = hash_secret(&password)?;
    let answer_hash = hash_secret(&answer)?;

    config_manager.update(|c| {
        c.security.password_hash = Some(password_hash);
        c.security.security_question = Some(question.to_string());
        c.security.security_answer_hash = Some(answer_hash);
        c.security.failed_attempts = 0;
        c.security.lock_until = None;
    })
}

#[tauri::command]
pub fn verify_exit_password(
    config_manager: tauri::State<Arc<ConfigManager>>,
    runtime_flags: tauri::State<RuntimeFlags>,
    app_handle: tauri::AppHandle,
    password: String,
) -> Result<VerifyResult, String> {
    if runtime_flags.safe_mode {
        app_handle.exit(0);
        return Ok(VerifyResult {
            ok: true,
            locked: false,
            lock_remaining_seconds: 0,
            remaining_attempts: MAX_FAILED_ATTEMPTS,
        });
    }

    reset_lock_if_expired(&config_manager)?;
    let mut security = config_manager.get()?.security.clone();

    let lock_remaining = lock_remaining_seconds(&security.lock_until);
    if lock_remaining > 0 {
        return Ok(VerifyResult {
            ok: false,
            locked: true,
            lock_remaining_seconds: lock_remaining,
            remaining_attempts: 0,
        });
    }

    let password_hash = security
        .password_hash
        .clone()
        .ok_or_else(|| "未设置退出密码".to_string())?;
    let password = normalize_password(&password);
    if password.is_empty() {
        return Err("请输入退出密码".to_string());
    }

    if verify_secret(&password_hash, &password)? {
        security.failed_attempts = 0;
        security.lock_until = None;
        config_manager.update(|c| c.security = security.clone())?;
        app_handle.exit(0);
        return Ok(VerifyResult {
            ok: true,
            locked: false,
            lock_remaining_seconds: 0,
            remaining_attempts: MAX_FAILED_ATTEMPTS,
        });
    }

    security.failed_attempts += 1;
    let mut locked = false;
    let mut lock_remaining_seconds = 0;
    if security.failed_attempts >= MAX_FAILED_ATTEMPTS {
        let lock_until = Local::now() + Duration::minutes(LOCK_MINUTES);
        security.lock_until = Some(lock_until.to_rfc3339());
        locked = true;
        lock_remaining_seconds = (LOCK_MINUTES * 60) as u64;
    }

    let remaining_attempts = MAX_FAILED_ATTEMPTS.saturating_sub(security.failed_attempts);
    config_manager.update(|c| c.security = security)?;

    Ok(VerifyResult {
        ok: false,
        locked,
        lock_remaining_seconds,
        remaining_attempts,
    })
}

#[tauri::command]
pub fn reset_password(
    config_manager: tauri::State<Arc<ConfigManager>>,
    security_answer: String,
    new_password: String,
) -> Result<(), String> {
    let config = config_manager.get()?;
    let security = config.security.clone();
    let answer_hash = security
        .security_answer_hash
        .clone()
        .ok_or_else(|| "未设置密保问题".to_string())?;

    let normalized_answer = normalize_answer(&security_answer);
    if normalized_answer.is_empty() {
        return Err("请输入密保答案".to_string());
    }
    if !verify_secret(&answer_hash, &normalized_answer)? {
        return Err("密保答案不正确".to_string());
    }

    let new_password = normalize_password(&new_password);
    if new_password.len() < 4 {
        return Err("新密码长度至少4位".to_string());
    }
    let new_hash = hash_secret(&new_password)?;

    config_manager.update(|c| {
        c.security.password_hash = Some(new_hash);
        c.security.failed_attempts = 0;
        c.security.lock_until = None;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let secret = "test-secret";
        let hash = hash_secret(secret).unwrap();
        assert!(verify_secret(&hash, secret).unwrap());
        assert!(!verify_secret(&hash, "wrong").unwrap());
    }
}
