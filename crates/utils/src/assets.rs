use directories::ProjectDirs;
use rust_embed::RustEmbed;

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

pub fn asset_dir() -> std::path::PathBuf {
    let path = if let Some(override_dir) = std::env::var_os("VIBE_KANBAN_DATA_DIR") {
        std::path::PathBuf::from(override_dir)
    } else if cfg!(debug_assertions) {
        std::path::PathBuf::from(PROJECT_ROOT).join("../../dev_assets")
    } else {
        prod_asset_dir_path()
    };

    // Ensure the directory exists
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create asset directory");
    }

    path
    // ✔ macOS → ~/Library/Application Support/MyApp
    // ✔ Linux → ~/.local/share/myapp   (respects XDG_DATA_HOME)
    // ✔ Windows → %APPDATA%\Example\MyApp
}

pub fn prod_asset_dir_path() -> std::path::PathBuf {
    ProjectDirs::from("ai", "bloop", "vibe-kanban")
        .expect("OS didn't give us a home directory")
        .data_dir()
        .to_path_buf()
}

pub fn config_path() -> std::path::PathBuf {
    asset_dir().join("config.json")
}

pub fn profiles_path() -> std::path::PathBuf {
    asset_dir().join("profiles.json")
}

pub fn credentials_path() -> std::path::PathBuf {
    asset_dir().join("credentials.json")
}

pub fn trusted_keys_path() -> std::path::PathBuf {
    asset_dir().join("trusted_ed25519_public_keys.json")
}

pub fn server_signing_key_path() -> std::path::PathBuf {
    asset_dir().join("server_ed25519_signing_key")
}

pub fn relay_host_credentials_path() -> std::path::PathBuf {
    asset_dir().join("relay_host_credentials.json")
}

#[derive(RustEmbed)]
#[folder = "../../assets/sounds"]
pub struct SoundAssets;

#[derive(RustEmbed)]
#[folder = "../../assets/scripts"]
pub struct ScriptAssets;

#[cfg(test)]
mod tests {
    use super::*;

    const ENV_VAR: &str = "VIBE_KANBAN_DATA_DIR";

    // Run env-var-touching tests serially to avoid races within this test
    // binary. No other tests in this crate read VIBE_KANBAN_DATA_DIR, but
    // setting/unsetting it is process-global so we still serialize amongst
    // ourselves.
    fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    fn with_env_override<R>(value: Option<&std::path::Path>, f: impl FnOnce() -> R) -> R {
        let _guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let prior = std::env::var_os(ENV_VAR);
        unsafe {
            match value {
                Some(v) => std::env::set_var(ENV_VAR, v),
                None => std::env::remove_var(ENV_VAR),
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        unsafe {
            match prior {
                Some(v) => std::env::set_var(ENV_VAR, v),
                None => std::env::remove_var(ENV_VAR),
            }
        }
        match result {
            Ok(r) => r,
            Err(e) => std::panic::resume_unwind(e),
        }
    }

    #[test]
    fn asset_dir_uses_env_override_when_set() {
        let override_path = std::env::temp_dir().join(format!(
            "vibe-kanban-asset-dir-test-{}",
            uuid::Uuid::new_v4()
        ));
        assert!(!override_path.exists());

        let returned = with_env_override(Some(&override_path), asset_dir);

        assert_eq!(returned, override_path);
        assert!(override_path.is_dir());
        let _ = std::fs::remove_dir_all(&override_path);
    }

    #[test]
    fn asset_dir_falls_back_when_env_unset() {
        let returned = with_env_override(None, asset_dir);

        let expected = if cfg!(debug_assertions) {
            std::path::PathBuf::from(PROJECT_ROOT).join("../../dev_assets")
        } else {
            prod_asset_dir_path()
        };
        assert_eq!(returned, expected);
    }
}
