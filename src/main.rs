use clap::Parser;

use cli::Cli;
use defguard_wireguard_rs::{Kernel, WGApi, WireguardInterfaceApi};
use serde_json::json;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path::Path;
use utils::send_signal_to_waybar;

mod cli;
mod config;
mod error;
mod utils;

#[derive(Copy, Clone)]
enum Status {
    Connected,
    Disconnected,
    Error,
}

impl Status {
    fn as_str(&self) -> &'static str {
        match self {
            Status::Connected => "connected",
            Status::Disconnected => "disconnected",
            Status::Error => "error",
        }
    }
    fn percentage(&self) -> u8 {
        match self {
            Status::Connected => 0,
            Status::Disconnected => 50,
            Status::Error => 100,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LastStateError {
    error: Option<HashMap<String, String>>,
}

fn status(interface_name: &str, state_filepath: std::path::PathBuf) -> Result<(), error::Error> {
    let bytes = std::fs::read(state_filepath)?;
    let error: LastStateError = serde_json::from_slice(&bytes)?;

    if let Some(e) = error.error {
        for (key, value) in e.iter() {
            if key == interface_name {
                output_json(
                    "VPN: Error",
                    Status::Error,
                    &format!("Toggle failed: {}", value),
                )?;
            }
        }
    }

    match WGApi::<Kernel>::new(interface_name.to_string()) {
        Ok(wg_api) => {
            let status = if wg_api.read_interface_data().is_ok() {
                Status::Connected
            } else {
                Status::Disconnected
            };
            output_json(
                &format!("VPN: {}", interface_name),
                status,
                &format!("VPN is {}", status.as_str()),
            )?;
        }
        Err(e) => {
            let err = error::Error::WireGuardApi(e.to_string());
            output_json(
                "VPN: Error",
                Status::Error,
                &format!("Failed to check VPN status: {}", err),
            )?;
        }
    }
    Ok(())
}
fn output_json(text: &str, status: Status, tooltip: &str) -> Result<(), std::io::Error> {
    let output = json!({
        "text": text,
        "class": status.as_str(),
        "tooltip": tooltip,
        "percentage": status.percentage()
    });
    println!("{}", output);
    io::stdout().flush()
}

fn toggle(
    interface_name: &str,
    config_path: &Path,
    signal_num: i32,
    state_filepath: std::path::PathBuf,
    debug: bool,
    port: u32,
) -> Result<(), error::Error> {
    let result = match WGApi::<Kernel>::new(interface_name.to_string()) {
        Ok(wg_api) => {
            let is_active = wg_api.read_interface_data().is_ok();
            if is_active {
                wg_api
                    .remove_interface()
                    .map_err(|e| error::Error::WireGuardApi(e.to_string()))
            } else {
                match config::configure_wireguard(config_path, interface_name, port) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if let error::Error::WireGuardApi(_) = e {
                            wg_api.remove_interface()?;
                            Err(error::Error::WireGuardApi(e.to_string()))
                        } else {
                            Err(e)
                        }
                    }
                }
            }
        }
        Err(e) => Err(error::Error::WireGuardApi(e.to_string())),
    };

    match result {
        Ok(_) => {
            utils::fs_write(state_filepath, "{}")?;
        }
        Err(e) => {
            let json_str = serde_json::to_string(&json!({
                "error": {interface_name: e.to_string()}
            }))?;
            utils::fs_write(state_filepath, json_str)?;
        }
    }
    send_signal_to_waybar(signal_num, debug)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config_path = Path::new(&cli.config);
    let interface_name = config_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| error::Error::InvalidFormat {
            message: "Invalid config file name".to_string(),
        });
    let interface_name = match interface_name {
        Ok(name) => name,
        Err(e) => {
            let err = e.to_string();
            output_json(
                "VPN: Error",
                Status::Error,
                &format!("Failed to parse interface name: {}", err),
            )?;
            return Err(Box::new(e));
        }
    };
    let state_home = utils::get_state_home("wg-waybar")?;
    if !state_home.exists() {
        utils::fs_create_dir(state_home.clone())?;
    }
    let state_filepath = state_home.join(cli.state_filename);
    if !state_filepath.exists() {
        utils::fs_write(state_filepath.clone(), "{}")?;
    }
    match &cli.command {
        Some(cli::Commands::Toggle) => toggle(
            interface_name,
            config_path,
            cli.signal,
            state_filepath,
            cli.debug,
            cli.port,
        )?,

        None => status(interface_name, state_filepath)?,
    };
    Ok(())
}
