use crate::error;
use base64::prelude::*;
use defguard_wireguard_rs::key::Key;
use defguard_wireguard_rs::net::IpAddrMask;
use defguard_wireguard_rs::{InterfaceConfiguration, host::Peer};
use defguard_wireguard_rs::{Kernel, WGApi, WireguardInterfaceApi};
use ini::{Ini, Properties};
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::str::FromStr;
use x25519_dalek::PublicKey;

#[derive(Debug)]
struct WireGuardConfig {
    interface: InterfaceConfig,
    peers: Vec<PeerConfig>,
}

struct InterfaceConfig {
    private_key: String,
    addresses: Vec<String>,
    dns: Option<Vec<String>>,
    listen_port: Option<u32>,
}
impl std::fmt::Debug for InterfaceConfig {
    // To avoid debugging private_key
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterfaceConfiguration")
            .field("name", &self.addresses)
            .field("addresses", &self.addresses)
            .field("listen_port", &self.listen_port)
            .finish_non_exhaustive()
    }
}

impl InterfaceConfig {
    fn load(properties: &Properties) -> Result<Self, error::Error> {
        let private_key = properties
            .get("PrivateKey")
            .ok_or_else(|| error::MissingPropertyError("PrivateKey is missing".into()))?
            .to_string();

        let addresses = properties
            .get("Address")
            .ok_or_else(|| error::MissingPropertyError("Address is missing".into()))?
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if !s.contains('/') {
                    return Err(error::Error::InvalidFormat {
                        message: format!("Invalid Address format: {}", s),
                    });
                }
                let parts: Vec<&str> = s.split('/').collect();
                IpAddr::from_str(parts[0]).map_err(|_| error::Error::InvalidFormat {
                    message: format!("Invalid IP in Address: {}", parts[0]),
                })?;
                Ok(s.to_string())
            })
            .collect::<Result<Vec<String>, error::Error>>()?;

        if addresses.is_empty() {
            return Err(error::Error::MissingProperty(error::MissingPropertyError(
                "Address cannot be empty".into(),
            )));
        }

        let dns = properties
            .get("DNS")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect());

        let listen_port = properties
            .get("ListenPort")
            .map(|port| {
                port.parse::<u32>()
                    .map_err(|_| error::Error::InvalidFormat {
                        message: format!("Invalid ListenPort: {}", port),
                    })
            })
            .transpose()?;

        Ok(Self {
            private_key,
            addresses,
            dns,
            listen_port,
        })
    }
}

#[derive(Debug)]
struct PeerConfig {
    public_key: PublicKey,
    endpoint: Option<SocketAddr>,
    allowed_ips: Vec<String>,
}

impl PeerConfig {
    fn load(properties: &Properties) -> Result<Self, error::Error> {
        let public_key_str = properties
            .get("PublicKey")
            .ok_or_else(|| error::MissingPropertyError("PublicKey is missing".into()))?;

        let public_key_bytes = BASE64_STANDARD
            .decode(public_key_str)
            .map_err(error::Error::Base64)?;

        let public_key_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
            error::Error::PeerConfig(error::PeerConfigError::InvalidPublicKey {
                message: "Public key must be 32 bytes".to_string(),
            })
        })?;

        let public_key = PublicKey::from(public_key_array);

        let endpoint = properties
            .get("Endpoint")
            .map(|e| SocketAddr::from_str(e).map_err(error::PeerConfigError::EndPoint))
            .transpose()?;

        let allowed_ips = properties
            .get("AllowedIPs")
            .ok_or_else(|| error::MissingPropertyError("AllowedIPs is missing".into()))?
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if !s.contains('/') {
                    return Err(error::Error::InvalidFormat {
                        message: format!("Invalid AllowedIPs format: {}", s),
                    });
                }
                let parts: Vec<&str> = s.split('/').collect();
                IpAddr::from_str(parts[0]).map_err(|_| error::Error::InvalidFormat {
                    message: format!("Invalid IP in AllowedIPs: {}", parts[0]),
                })?;
                Ok(s.to_string())
            })
            .collect::<Result<Vec<String>, error::Error>>()?;

        if allowed_ips.is_empty() {
            return Err(error::Error::MissingProperty(error::MissingPropertyError(
                "AllowedIPs cannot be empty".into(),
            )));
        }

        Ok(Self {
            public_key,
            endpoint,
            allowed_ips,
        })
    }
}

fn parse_wg_config(file_path: &Path) -> Result<WireGuardConfig, error::Error> {
    let conf_str = fs::read_to_string(file_path)?;
    let conf = Ini::load_from_str(&conf_str)?;

    let interface_section = conf
        .section(Some("Interface"))
        .ok_or(error::MissingSectionError("Interface".into()))?;

    let interface_config = InterfaceConfig::load(interface_section)?;

    let mut peers = Vec::new();
    for (section_name, section) in conf.iter() {
        if section_name.unwrap_or_default().starts_with("Peer") {
            let peer_config = PeerConfig::load(section)?;
            peers.push(peer_config);
        }
    }

    Ok(WireGuardConfig {
        interface: interface_config,
        peers,
    })
}

fn parse_ip_addr_mask(addr: &str) -> Result<IpAddrMask, error::Error> {
    let parts: Vec<&str> = addr.split('/').collect();
    if parts.len() != 2 {
        return Err(error::Error::InvalidFormat {
            message: format!("Invalid IP/CIDR format: {}", addr),
        });
    }
    let ip = IpAddr::from_str(parts[0]).map_err(|_| error::Error::InvalidFormat {
        message: format!("Invalid IP: {}", parts[0]),
    })?;
    let cidr = parts[1]
        .parse::<u8>()
        .map_err(|_| error::Error::InvalidFormat {
            message: format!("Invalid CIDR prefix: {}", parts[1]),
        })?;
    Ok(IpAddrMask::new(ip, cidr))
}

pub fn configure_wireguard(config_path: &Path, interface_name: &str, port: u32) -> Result<(), error::Error> {
    let wg_config = parse_wg_config(config_path)?;
    let wg_api = WGApi::<Kernel>::new(interface_name.to_string())?;
    wg_api.create_interface()?;

    let addresses = wg_config
        .interface
        .addresses
        .iter()
        .map(|addr| parse_ip_addr_mask(addr))
        .collect::<Result<Vec<IpAddrMask>, error::Error>>()?;

    let config = InterfaceConfiguration {
        name: interface_name.to_string(),
        prvkey: wg_config.interface.private_key,
        addresses,
        port: wg_config.interface.listen_port.unwrap_or(port),
        peers: vec![],
        mtu: None,
    };
    wg_api.configure_interface(&config)?;

    if let Some(dns) = wg_config.interface.dns {
        let dns_ips = dns
            .iter()
            .map(|d| {
                IpAddr::from_str(d).map_err(|e| error::Error::InvalidFormat {
                    message: format!("Invalid DNS IP: {}", e),
                })
            })
            .collect::<Result<Vec<IpAddr>, error::Error>>()?;
        wg_api.configure_dns(&dns_ips, &[])?;
    }

    for peer in wg_config.peers {
        let public_key_bytes = *peer.public_key.as_bytes();
        let key = Key::new(public_key_bytes);
        let mut peer_config = Peer::new(key);

        let allowed_ips = peer
            .allowed_ips
            .iter()
            .map(|ip| parse_ip_addr_mask(ip))
            .collect::<Result<Vec<IpAddrMask>, error::Error>>()?;
        peer_config.set_allowed_ips(allowed_ips);

        if let Some(endpoint) = peer.endpoint {
            peer_config.set_endpoint(&endpoint.to_string())?;
        }

        wg_api.configure_peer(&peer_config)?;
    }

    Ok(())
}
