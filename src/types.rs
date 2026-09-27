use serde::{Deserialize, Serialize};

use foundation::serialization::{JsonDocument, JsonObject};

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

impl ServiceType {
    /// Wire spelling used on the daemon socket (serde variant names).
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            ServiceType::Sys => "Sys",
            ServiceType::Default => "Default",
            ServiceType::Low => "Low",
        }
    }

    pub fn from_wire_str(s: &str) -> Option<Self> {
        match s {
            "Sys" => Some(ServiceType::Sys),
            "Default" => Some(ServiceType::Default),
            "Low" => Some(ServiceType::Low),
            _ => None,
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

impl ServiceState {
    /// Wire spelling used on the daemon socket (serde variant names).
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            ServiceState::Stopped => "Stopped",
            ServiceState::Starting => "Starting",
            ServiceState::Running => "Running",
            ServiceState::Deactivated => "Deactivated",
            ServiceState::Crashed => "Crashed",
        }
    }

    pub fn from_wire_str(s: &str) -> Option<Self> {
        match s {
            "Stopped" => Some(ServiceState::Stopped),
            "Starting" => Some(ServiceState::Starting),
            "Running" => Some(ServiceState::Running),
            "Deactivated" => Some(ServiceState::Deactivated),
            "Crashed" => Some(ServiceState::Crashed),
            _ => None,
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

fn err_missing(field: &str) -> String {
    format!("missing field `{}`", field)
}

impl ServiceInfo {
    pub fn from_document(doc: &JsonDocument) -> Result<Self, String> {
        let name = doc
            .str_field("name")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| err_missing("name"))?;
        let service_type = doc
            .str_field("service_type")
            .map_err(|e| e.to_string())?
            .map(|v| {
                ServiceType::from_wire_str(&v)
                    .ok_or_else(|| format!("unknown service_type `{}`", v))
            })
            .transpose()?
            .ok_or_else(|| err_missing("service_type"))?;
        let state = doc
            .str_field("state")
            .map_err(|e| e.to_string())?
            .map(|v| {
                ServiceState::from_wire_str(&v).ok_or_else(|| format!("unknown state `{}`", v))
            })
            .transpose()?
            .ok_or_else(|| err_missing("state"))?;
        let user = doc
            .str_field("user")
            .map_err(|e| e.to_string())?
            .ok_or_else(|| err_missing("user"))?;
        Ok(Self {
            name,
            service_type,
            state,
            pid: doc
                .u64_field("pid")
                .map_err(|e| e.to_string())?
                .map(|v| u32::try_from(v).map_err(|_| format!("pid out of range: {}", v)))
                .transpose()?,
            user,
        })
    }

    pub fn to_json_string(&self) -> String {
        let mut obj = JsonObject::new();
        obj.field_str("name", &self.name);
        obj.field_str("service_type", self.service_type.as_wire_str());
        obj.field_str("state", self.state.as_wire_str());
        match self.pid {
            Some(pid) => {
                obj.field_u64("pid", pid as u64);
            }
            None => {
                obj.field_null("pid");
            }
        }
        obj.field_str("user", &self.user);
        obj.build(false).unwrap_or_else(|_| "{}".to_string())
    }
}

fn build_options(options: &IpcOptions) -> String {
    let mut obj = JsonObject::new();
    match options.head {
        Some(head) => {
            obj.field_u64("head", head as u64);
        }
        None => {
            obj.field_null("head");
        }
    }
    match options.app_path.as_deref() {
        Some(path) => {
            obj.field_str("app_path", path);
        }
        None => {
            obj.field_null("app_path");
        }
    }
    obj.build(false).unwrap_or_else(|_| "{}".to_string())
}

fn parse_options(doc: &JsonDocument) -> Result<IpcOptions, String> {
    let options = doc
        .nested("options")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(JsonDocument::empty);
    Ok(IpcOptions {
        head: options
            .u64_field("head")
            .map_err(|e| e.to_string())?
            .map(|v| {
                usize::try_from(v).map_err(|_| format!("head out of range: {}", v))
            })
            .transpose()?,
        app_path: options
            .str_field("app_path")
            .map_err(|e| e.to_string())?,
    })
}

impl IpcRequest {
    pub fn to_json_string(&self) -> String {
        let mut obj = JsonObject::new();
        obj.field_str("action", &self.action);
        match self.service.as_deref() {
            Some(service) => {
                obj.field_str("service", service);
            }
            None => {
                obj.field_null("service");
            }
        }
        let _ = obj.field_raw("options", &build_options(&self.options));
        obj.build(false).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json_str(s: &str) -> Result<Self, String> {
        let doc = JsonDocument::parse(s).map_err(|e| e.to_string())?;
        Ok(Self {
            action: doc
                .str_field("action")
                .map_err(|e| e.to_string())?
                .ok_or_else(|| err_missing("action"))?,
            service: doc.str_field("service").map_err(|e| e.to_string())?,
            options: parse_options(&doc)?,
        })
    }
}

impl IpcResponse {
    pub fn to_json_string(&self) -> String {
        let mut obj = JsonObject::new();
        obj.field_bool("success", self.success);
        match self.message.as_deref() {
            Some(message) => {
                obj.field_str("message", message);
            }
            None => {
                obj.field_null("message");
            }
        }
        match &self.data {
            Some(data) => {
                let items: Vec<String> =
                    data.services.iter().map(ServiceInfo::to_json_string).collect();
                let _ = obj.field_raw("data", &format!("{{\"services\": [{}]}}", items.join(",")));
            }
            None => {
                obj.field_null("data");
            }
        }
        obj.build(false).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json_str(s: &str) -> Result<Self, String> {
        let doc = JsonDocument::parse(s).map_err(|e| e.to_string())?;
        let data = doc
            .nested("data")
            .map_err(|e| e.to_string())?
            .map(|data| {
                data.array_field("services")
                    .map_err(|e| e.to_string())?
                    .iter()
                    .map(ServiceInfo::from_document)
                    .collect::<Result<Vec<_>, _>>()
                    .map(|services| IpcData { services })
            })
            .transpose()?;
        Ok(Self {
            success: doc
                .bool_field("success")
                .map_err(|e| e.to_string())?
                .ok_or_else(|| err_missing("success"))?,
            message: doc.str_field("message").map_err(|e| e.to_string())?,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_spellings_match_serde() {
        assert_eq!(ServiceType::Sys.as_wire_str(), "Sys");
        assert_eq!(ServiceType::Default.as_wire_str(), "Default");
        assert_eq!(ServiceType::Low.as_wire_str(), "Low");
        assert_eq!(ServiceState::Running.as_wire_str(), "Running");
        assert_eq!(ServiceState::Deactivated.as_wire_str(), "Deactivated");
        assert_eq!(ServiceType::from_wire_str("Sys"), Some(ServiceType::Sys));
        assert_eq!(ServiceType::from_wire_str("sys"), None);
    }

    #[test]
    fn request_roundtrip() {
        let req = IpcRequest {
            action: "start".to_string(),
            service: Some("test".to_string()),
            options: IpcOptions { head: Some(10), app_path: None },
        };
        let back = IpcRequest::from_json_str(&req.to_json_string()).unwrap();
        assert_eq!(back.action, "start");
        assert_eq!(back.service.as_deref(), Some("test"));
        assert_eq!(back.options.head, Some(10));
        assert_eq!(back.options.app_path, None);
    }

    #[test]
    fn response_roundtrip() {
        let resp = IpcResponse {
            success: true,
            message: None,
            data: Some(IpcData {
                services: vec![ServiceInfo {
                    name: "svc".to_string(),
                    service_type: ServiceType::Sys,
                    state: ServiceState::Running,
                    pid: Some(42),
                    user: "root".to_string(),
                }],
            }),
        };
        let back = IpcResponse::from_json_str(&resp.to_json_string()).unwrap();
        assert!(back.success);
        let svc = &back.data.unwrap().services[0];
        assert_eq!(svc.service_type, ServiceType::Sys);
        assert_eq!(svc.state, ServiceState::Running);
        assert_eq!(svc.pid, Some(42));
    }
}
