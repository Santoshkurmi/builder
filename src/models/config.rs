use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub log_path: String,
    pub ssl: SslConfig,
    pub auth: AuthConfig,
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SslConfig {
    pub enable_ssl: bool,
    pub certificate_path: String,
    pub certificate_key_path: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthType {
    Token,
    Address,
    Both,
}

impl std::str::FromStr for AuthType {
    fn from_str(input: &str) -> AuthType {
        match input {
            "token" => Ok(AuthType::Token),
            "address" => Ok(AuthType::Address),
            "both" => Ok(AuthType::Both),
            //_ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AddressType {
    IP,
    Hostname,
}

impl std::str::FromStr for AddressType {
    fn from_str(input: &str) -> AddressType {
        match input {
            "ip" => Ok(AddressType::IP),
            "hostname" => Ok(AddressType::hostname),
            //_ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,       // "token", "address", "both"
    pub address_type: AddressType, // "ip", "hostname"
    pub allowed_addresses: Vec<String>,
    pub allowed_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub allow_multi_build: bool,
    pub max_pending_build: u32,
    pub base_endpoint_path: String,
    pub api: ApiConfig,
    pub build: BuildConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payload {
    pub r#type: PayloadType,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PayloadType {
    Normal,
    Env,
    Param,
    File,
}

impl std::str::FromStr for PayloadType {
    fn from_str(input: &str) -> PayloadType {
        match input {
            "normal" => Ok(PayloadType::Normal),
            "env" => Ok(PayloadType::Env),
            "param" => Ok(PayloadType::Param),
            "file" => Ok(PayloadType::File),
            //_ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildConfig {
    pub project_path: String,
    pub payload: Vec<Payload>,
    pub unique_build_key: String,
    pub on_success_failure: String,
    pub on_success_error_payload: Vec<String>,
    pub commands: Vec<CommandConfig>,
    #[serde(default)]
    pub run_on_success: Vec<CommandConfig>,
    #[serde(default)]
    pub run_on_failure: Vec<CommandConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandConfig {
    pub command: String,
    pub title: String,
    #[serde(default)]
    pub payload: Vec<Payload>,
    #[serde(default)]
    pub on_error: String, // "abort", "continue"
    #[serde(default = true)]
    pub send_to_sock: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
