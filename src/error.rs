use ini::ParseError;
use std::net::AddrParseError;
#[derive(Debug)]
pub struct MissingSectionError(pub String);

#[derive(Debug)]
pub struct MissingPropertyError(pub String);

impl std::error::Error for MissingPropertyError {}

impl std::fmt::Display for MissingPropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing property: {}", self.0)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Ini(ParseError),
    MissingSection(MissingSectionError),
    MissingProperty(MissingPropertyError),
    PeerConfig(PeerConfigError),
    Signal(SignalError),
    InvalidFormat { message: String },
    WireGuardApi(String),
    Base64(base64::DecodeError),
    UserNotFound(String),
    Serde(serde_json::error::Error),
    UnCaught(UnCaughtError),
}

#[derive(Debug)]
pub struct UnCaughtError(pub String);
impl std::error::Error for UnCaughtError {}

impl std::fmt::Display for UnCaughtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<UnCaughtError> for Error {
    fn from(value: UnCaughtError) -> Self {
        Self::UnCaught(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(err) => write!(f, "I/O error: {}", err),
            Error::Ini(err) => write!(f, "INI parsing error: {}", err),
            Error::MissingSection(err) => write!(f, "{}", err),
            Error::MissingProperty(err) => write!(f, "{}", err),
            Error::PeerConfig(err) => write!(f, "{}", err),
            Error::InvalidFormat { message } => write!(f, "Invalid format: {}", message),
            Error::WireGuardApi(err) => write!(f, "WireGuard API error: {}", err),
            Error::Base64(err) => write!(f, "Base64 decoding error: {}", err),
            Error::UserNotFound(err) => write!(f, "UserNotFound error: {}", err),
            Error::Serde(err) => write!(f, "SerdeError: {}", err),
            Error::Signal(err) => write!(f, "SignalError: {}", err),
            Error::UnCaught(err) => write!(f, "UnCaughtError: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            Error::Ini(err) => Some(err),
            Error::MissingSection(err) => Some(err),
            Error::MissingProperty(err) => Some(err),
            Error::PeerConfig(err) => Some(err),
            Error::Base64(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<SignalError> for Error {
    fn from(value: SignalError) -> Self {
        Self::Signal(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Ini(value)
    }
}

impl From<MissingSectionError> for Error {
    fn from(value: MissingSectionError) -> Self {
        Self::MissingSection(value)
    }
}

impl From<MissingPropertyError> for Error {
    fn from(value: MissingPropertyError) -> Self {
        Self::MissingProperty(value)
    }
}

impl From<PeerConfigError> for Error {
    fn from(value: PeerConfigError) -> Self {
        Self::PeerConfig(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<defguard_wireguard_rs::error::WireguardInterfaceError> for Error {
    fn from(value: defguard_wireguard_rs::error::WireguardInterfaceError) -> Self {
        Self::WireGuardApi(value.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64(value)
    }
}

impl std::fmt::Display for MissingSectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Missing section {} in WireGuard configuration file",
            self.0
        )
    }
}

impl std::error::Error for MissingSectionError {}

#[derive(Debug)]
pub enum PeerConfigError {
    EndPoint(AddrParseError),
    MissingProperty(MissingPropertyError),
    InvalidPublicKey { message: String },
}

impl std::error::Error for PeerConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PeerConfigError::EndPoint(err) => Some(err),
            PeerConfigError::MissingProperty(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for PeerConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerConfigError::EndPoint(err) => write!(f, "Endpoint parsing error: {}", err),
            PeerConfigError::MissingProperty(err) => write!(f, "{}", err),
            PeerConfigError::InvalidPublicKey { message } => {
                write!(f, "Invalid public key: {}", message)
            }
        }
    }
}

impl From<MissingPropertyError> for PeerConfigError {
    fn from(value: MissingPropertyError) -> Self {
        Self::MissingProperty(value)
    }
}

#[derive(Debug)]
pub enum SignalError {
    OutOfRange(SignalOutOfRangeError),
    ProcessNotFound(ProcessNotFoundError),
    OS(String),
}

impl std::error::Error for SignalError {}

#[derive(Debug)]
pub struct SignalOutOfRangeError(pub String);
#[derive(Debug)]
pub struct ProcessNotFoundError(pub String);

impl std::fmt::Display for SignalOutOfRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signal out of allowed range: {}", self.0)
    }
}
impl std::fmt::Display for ProcessNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Process not found: {}", self.0)
    }
}

impl std::error::Error for SignalOutOfRangeError {}
impl std::error::Error for ProcessNotFoundError {}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalError::OutOfRange(v) => write!(f, "{}", v),
            SignalError::ProcessNotFound(v) => write!(f, "{}", v),
            SignalError::OS(v) => write!(f, "OS error: {}", v),
        }
    }
}

impl From<ProcessNotFoundError> for SignalError {
    fn from(value: ProcessNotFoundError) -> Self {
        Self::ProcessNotFound(value)
    }
}

impl From<SignalOutOfRangeError> for SignalError {
    fn from(value: SignalOutOfRangeError) -> Self {
        Self::OutOfRange(value)
    }
}
