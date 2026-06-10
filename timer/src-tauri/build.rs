use std::env;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"))
}

fn local_activation_env_path() -> PathBuf {
    manifest_dir()
        .join("..")
        .join("..")
        .join("config")
        .join("local")
        .join("activation.env")
}

fn parse_env_file(path: &PathBuf) -> Vec<(String, String)> {
    let content = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn lookup_value(entries: &[(String, String)], key: &str) -> Option<String> {
    entries
        .iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value.clone())
}

fn resolve_value(key: &str, file_entries: &[(String, String)]) -> Option<String> {
    if let Ok(value) = env::var(key) {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    lookup_value(file_entries, key)
}

fn validate_secret_hex(secret_hex: &str) -> Result<(), String> {
    let normalized = secret_hex.trim();
    if normalized.len() != 64 {
        return Err(format!(
            "{key} must be 64 hex chars (32 bytes), got {len}",
            key = "TIMER_ACTIVATION_SECRET_HEX",
            len = normalized.len()
        ));
    }
    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("TIMER_ACTIVATION_SECRET_HEX must contain only hex characters".to_string());
    }
    Ok(())
}

fn main() {
    let local_env_path = local_activation_env_path();
    println!("cargo:rerun-if-changed={}", local_env_path.display());
    println!("cargo:rerun-if-env-changed=TIMER_ACTIVATION_SECRET_HEX");
    println!("cargo:rerun-if-env-changed=TIMER_GENERATOR_PASSWORD");

    let file_entries = parse_env_file(&local_env_path);

    let secret_hex = resolve_value("TIMER_ACTIVATION_SECRET_HEX", &file_entries).unwrap_or_default();
    let generator_password =
        resolve_value("TIMER_GENERATOR_PASSWORD", &file_entries).unwrap_or_default();

    if let Err(err) = validate_secret_hex(&secret_hex) {
        panic!(
            "{err}\n\
             Configure build secrets before compiling:\n\
             1) Copy config/public/activation.env.example to config/local/activation.env\n\
             2) Or export TIMER_ACTIVATION_SECRET_HEX / TIMER_GENERATOR_PASSWORD\n\
             See docs/release/CONFIGURATION.md"
        );
    }

    if generator_password.trim().is_empty() {
        panic!(
            "Missing TIMER_GENERATOR_PASSWORD.\n\
             Copy config/public/activation.env.example to config/local/activation.env\n\
             See docs/release/CONFIGURATION.md"
        );
    }

    println!("cargo:rustc-env=TIMER_ACTIVATION_SECRET_HEX={secret_hex}");
    println!(
        "cargo:rustc-env=TIMER_GENERATOR_PASSWORD={}",
        generator_password.trim()
    );

    tauri_build::build()
}
