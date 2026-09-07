use serde::{Deserialize, Serialize};

pub const SOCKET_PATH: &str = "/run/launchpad.sock";
pub const LAUNCHPAD_DIR: &str = "/Library/System/Launchpads";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    Sys,
    Default,
    Low,
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Sys => write!(f, "sys"),
            ServiceType::Default => write!(f, "default"),
            ServiceType::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Deactivated,
    Crashed,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Stopped => write!(f, "stopped"),
            ServiceState::Starting => write!(f, "starting"),
            ServiceState::Running => write!(f, "running"),
            ServiceState::Deactivated => write!(f, "deactivated"),
            ServiceState::Crashed => write!(f, "crashed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub execute: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub user: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default = "default_restart")]
    pub restart: bool,
}

fn default_restart() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub service_type: ServiceType,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub action: String,
    pub service: Option<String>,
    #[serde(default)]
    pub options: IpcOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpcOptions {
    pub head: Option<usize>,
    pub app_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<IpcData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcData {
    #[serde(default)]
    pub services: Vec<ServiceInfo>,
}
