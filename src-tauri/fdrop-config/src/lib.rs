use fdrop_common::human_readable_error;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use tauri::{AppHandle, Manager};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("The data directory {path} could not be created")]
    DataDirNotCreated {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("FDrop folder at {path} could not be created")]
    FDRopDirNotCreated {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to save key pair")]
    KeyWriteError(std::io::Error),
    #[error("Failed to save key pair")]
    KeyReadError(std::io::Error),
    #[error("Invalid bytes present in identity file")]
    InvalidIdentityBytes,
    #[error("An IO operation failed")]
    Io(#[from] std::io::Error),
    #[error("Failed to resolve the local application data folder")]
    DataDirUnresolved,
    #[error("invalid json config")]
    InvalidConfig,
}

const CONFIGFILE: &'static str = "config.json";

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    pub user: String,
    pub instance_name: String,
    pub fdrop_dir: PathBuf,
}

pub fn data_dir(handle: &AppHandle) -> tauri::Result<PathBuf> {
    handle.path().app_local_data_dir()
}

pub fn read_keys(handle: &AppHandle) -> Result<libp2p_identity::ed25519::Keypair, ConfigError> {
    data_dir(handle)
        .map_err(|_| ConfigError::DataDirUnresolved)
        .and_then(|f| {
            let mut identity = [0u8; 64];
            let mut identity_path = f.clone();
            identity_path.push("identity");
            File::open(&identity_path)?
                .read_exact(&mut identity)
                .map_err(|e| ConfigError::KeyReadError(e))?;
            libp2p_identity::ed25519::Keypair::try_from_bytes(&mut identity)
                .map_err(|_| ConfigError::InvalidIdentityBytes)
        })
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn check_first_launch(handle: AppHandle) -> bool {
        let configfile = handle.path().app_local_data_dir().and_then(|mut d| {
            d.push(CONFIGFILE);
            Ok(d)
        });

        configfile.is_ok() && configfile.unwrap().exists()
    }

    #[tauri::command]
    pub async fn initial_setup(handle: AppHandle, config: UserConfig) -> Result<(), String> {
        let configfile = data_dir(&handle)
            .and_then(|mut d| {
                d.push(CONFIGFILE);
                Ok(d)
            })
            .map_err(|_| ConfigError::DataDirUnresolved.to_string())?;
        let json_config = serde_json::to_string(&config).unwrap();
        let mut file = File::create(configfile).map_err(|e| human_readable_error(&e))?;

        file.write_all(json_config.as_bytes())
            .map_err(|e| human_readable_error(&e))?;

        std::fs::create_dir(&config.fdrop_dir).map_err(|e| {
            let start = ConfigError::FDRopDirNotCreated {
                path: config.fdrop_dir.clone(),
                source: e,
            };
            human_readable_error(&start)
        })?;

        Ok(())
    }

    #[tauri::command]
    pub fn get_device_details(handle: AppHandle) -> UserConfig {
        let hostname = whoami::fallible::hostname().unwrap_or(String::new());
        let fdrop_dir = handle
            .path()
            .home_dir()
            .and_then(|mut home| {
                home.push("FDrop");
                Ok(home)
            })
            .unwrap_or(PathBuf::from(""));

        UserConfig {
            instance_name: hostname,
            user: whoami::realname(),
            fdrop_dir,
        }
    }

    #[tauri::command]
    pub fn generate_keys(handle: AppHandle) -> Result<(), String> {
        let keypair = libp2p_identity::ed25519::Keypair::generate();
        let data_dir = data_dir(&handle).map_err(|_| ConfigError::DataDirUnresolved.to_string())?;
        let mut identity = data_dir.clone();
        identity.push("identity");
        File::create(identity)
            .and_then(|mut f| f.write_all(&keypair.to_bytes()))
            .map_err(|e| human_readable_error(&ConfigError::KeyWriteError(e)))?;

        Ok(())
    }
}
