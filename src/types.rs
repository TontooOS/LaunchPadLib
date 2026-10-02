use foundation::serialization::{JsonDocument, JsonObject, JsonValue};
use foundation::yaml;

pub const SOCKET_PATH: &str = "/run/launchpad.sock";
pub const LAUNCHPAD_DIR: &str = "/Library/System/Launchpads";

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Wire spelling used on the daemon socket.
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

    /// Spelling used in `.service` files. Case-insensitive, so both `sys`
    /// and `Sys` are accepted.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "sys" => Some(ServiceType::Sys),
            "default" => Some(ServiceType::Default),
            "low" => Some(ServiceType::Low),
            _ => None,
        }
    }

    /// Spelling written by [`ServiceConfig::to_yaml_string`].
    pub fn as_config_str(&self) -> &'static str {
        match self {
            ServiceType::Sys => "sys",
            ServiceType::Default => "default",
            ServiceType::Low => "low",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Wire spelling used on the daemon socket.
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

/// One `.service` definition file.
///
/// Parsed from YAML with [`ServiceConfig::from_yaml_str`], which uses
/// Foundation's YAML reader. No derive macros are involved.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub execute: String,
    pub service_type: ServiceType,
    pub user: String,
    pub depends_on: Vec<String>,
    pub restart: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            execute: String::new(),
            service_type: ServiceType::Default,
            user: String::new(),
            depends_on: Vec::new(),
            restart: true,
        }
    }
}

fn field<'a>(doc: &'a JsonValue, key: &str) -> Option<&'a JsonValue> {
    match doc.get(key) {
        None | Some(JsonValue::Null) => None,
        Some(value) => Some(value),
    }
}

fn need_str(doc: &JsonValue, key: &str) -> Result<String, String> {
    match field(doc, key) {
        None => Err(err_missing(key)),
        Some(JsonValue::Str(s)) => Ok(s.clone()),
        Some(_) => Err(format!("field `{key}` must be a string")),
    }
}

impl ServiceConfig {
    /// Parse a `.service` file body.
    ///
    /// ```rust
    /// use launchpad_lib::types::{ServiceConfig, ServiceType};
    ///
    /// let config = ServiceConfig::from_yaml_str(
    ///     "name: dock\nexecute: /bin/dock\ntype: sys\nuser: root\ndepends_on:\n  - compositor\n",
    /// )
    /// .unwrap();
    /// assert_eq!(config.service_type, ServiceType::Sys);
    /// assert_eq!(config.depends_on, vec!["compositor".to_string()]);
    /// ```
    ///
    /// - `name`, `execute`, `type` and `user` are required; a missing or
    ///   empty `name` / `execute` is an error (the daemon also checks this).
    /// - `type` is matched case-insensitively (`sys`, `default`, `low`).
    /// - `depends_on` defaults to an empty list.
    /// - `restart` defaults to `true`.
    /// - A `.service` file is a single document; more than one is an error.
    pub fn from_yaml_str(text: &str) -> Result<Self, String> {
        let doc = yaml::parse(text).map_err(|e| format!("YAML parse error: {}", e))?;
        if doc.is_null() {
            return Err("Service file is empty".to_string());
        }
        if !doc.is_object() {
            return Err("Service file must be a mapping".to_string());
        }

        let service_type = match field(&doc, "type") {
            None => return Err(err_missing("type")),
            Some(JsonValue::Str(s)) => ServiceType::from_config_str(s)
                .ok_or_else(|| format!("unknown service type `{}`", s))?,
            Some(_) => return Err("field `type` must be a string".to_string()),
        };

        let depends_on = match field(&doc, "depends_on") {
            None => Vec::new(),
            Some(JsonValue::Array(items)) => items
                .iter()
                .map(|item| match item {
                    JsonValue::Str(s) => Ok(s.clone()),
                    _ => Err("field `depends_on` must be a list of strings".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err("field `depends_on` must be a list".to_string()),
        };

        let restart = match field(&doc, "restart") {
            None => true,
            Some(JsonValue::Bool(b)) => *b,
            Some(JsonValue::Str(s)) => match s.as_str() {
                "true" | "yes" | "1" => true,
                "false" | "no" | "0" => false,
                other => return Err(format!("field `restart` is not a bool: {other}")),
            },
            Some(_) => return Err("field `restart` must be a bool".to_string()),
        };

        let config = Self {
            name: need_str(&doc, "name")?,
            execute: need_str(&doc, "execute")?,
            service_type,
            user: need_str(&doc, "user")?,
            depends_on,
            restart,
        };
        if config.name.is_empty() {
            return Err("Service name is empty".to_string());
        }
        if config.execute.is_empty() {
            return Err("Service execute path is empty".to_string());
        }
        Ok(config)
    }

    /// Render the config back to `.service` YAML. Round-trips through
    /// [`ServiceConfig::from_yaml_str`].
    pub fn to_yaml_string(&self) -> String {
        let deps = JsonValue::Array(
            self.depends_on
                .iter()
                .map(|name| JsonValue::Str(name.clone()))
                .collect(),
        );
        let doc = JsonValue::Object(vec![
            ("name".to_string(), JsonValue::Str(self.name.clone())),
            ("execute".to_string(), JsonValue::Str(self.execute.clone())),
            (
                "type".to_string(),
                JsonValue::Str(self.service_type.as_config_str().to_string()),
            ),
            ("user".to_string(), JsonValue::Str(self.user.clone())),
            ("depends_on".to_string(), deps),
            ("restart".to_string(), JsonValue::Bool(self.restart)),
        ]);
        yaml::to_yaml(&doc)
    }

    /// Read and parse a `.service` file from disk.
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        Self::from_yaml_str(&text)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub service_type: ServiceType,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct IpcRequest {
    pub action: String,
    pub service: Option<String>,
    pub options: IpcOptions,
}

#[derive(Debug, Clone, Default)]
pub struct IpcOptions {
    pub head: Option<usize>,
    pub app_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IpcResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<IpcData>,
}

#[derive(Debug, Clone)]
pub struct IpcData {
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
    fn wire_spellings_match_the_socket_protocol() {
        assert_eq!(ServiceType::Sys.as_wire_str(), "Sys");
        assert_eq!(ServiceType::Default.as_wire_str(), "Default");
        assert_eq!(ServiceType::Low.as_wire_str(), "Low");
        assert_eq!(ServiceState::Running.as_wire_str(), "Running");
        assert_eq!(ServiceState::Deactivated.as_wire_str(), "Deactivated");
        assert_eq!(ServiceType::from_wire_str("Sys"), Some(ServiceType::Sys));
        assert_eq!(ServiceType::from_wire_str("sys"), None);
    }

    #[test]
    fn config_spellings_are_case_insensitive() {
        assert_eq!(ServiceType::from_config_str("sys"), Some(ServiceType::Sys));
        assert_eq!(ServiceType::from_config_str("SYS"), Some(ServiceType::Sys));
        assert_eq!(
            ServiceType::from_config_str("Default"),
            Some(ServiceType::Default)
        );
        assert_eq!(ServiceType::from_config_str("low"), Some(ServiceType::Low));
        assert_eq!(ServiceType::from_config_str("nope"), None);
        assert_eq!(ServiceType::Sys.as_config_str(), "sys");
    }

    #[test]
    fn service_config_parses_a_real_service_file() {
        let text = "\
# TontooOS system dock (external app, replaces the former
# compositor-internal dock rendering).
name: dock
execute: /usr/local/bin/start-dock.sh
type: sys
user: liveuser
depends_on:
  - compositor
  - live-setup
restart: true
";
        let config = ServiceConfig::from_yaml_str(text).unwrap();
        assert_eq!(config.name, "dock");
        assert_eq!(config.execute, "/usr/local/bin/start-dock.sh");
        assert_eq!(config.service_type, ServiceType::Sys);
        assert_eq!(config.user, "liveuser");
        assert_eq!(
            config.depends_on,
            vec!["compositor".to_string(), "live-setup".to_string()]
        );
        assert!(config.restart);
    }

    #[test]
    fn service_config_defaults_match_the_old_serde_attributes() {
        let config =
            ServiceConfig::from_yaml_str("name: a\nexecute: /b\ntype: low\nuser: root\n").unwrap();
        assert!(config.depends_on.is_empty(), "depends_on defaulted to []");
        assert!(config.restart, "restart defaulted to true");

        let config = ServiceConfig::from_yaml_str(
            "name: a\nexecute: /b\ntype: sys\nuser: root\nrestart: false\ndepends_on: []\n",
        )
        .unwrap();
        assert!(!config.restart);
        assert!(config.depends_on.is_empty());
    }

    #[test]
    fn service_config_rejects_bad_input() {
        for bad in [
            "",
            "# only comments\n",
            "- a\n- b\n",
            "name: a\ntype: sys\nuser: root\n",             // no execute
            "execute: /b\ntype: sys\nuser: root\n",         // no name
            "name: a\nexecute: /b\nuser: root\n",           // no type
            "name: a\nexecute: /b\ntype: bogus\nuser: r\n", // bad type
            "name: a\nexecute: /b\ntype: sys\n",            // no user
            "name:\nexecute: /b\ntype: sys\nuser: root\n",  // empty name
            "name: a\nexecute:\ntype: sys\nuser: root\n",   // empty execute
            "name: a\nexecute: /b\ntype: sys\nuser: root\ndepends_on: notalist\n",
            "name: a\nexecute: /b\ntype: sys\nuser: root\nrestart: maybe\n",
            "name: 1\nexecute: /b\ntype: sys\nuser: root\n", // non-string name
            "---\nname: a\n---\nname: b\n",                    // two documents
        ] {
            assert!(
                ServiceConfig::from_yaml_str(bad).is_err(),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn service_config_yaml_roundtrip() {
        let config = ServiceConfig {
            name: "seatd".to_string(),
            execute: "/usr/bin/seatd -g seat".to_string(),
            service_type: ServiceType::Low,
            user: "root".to_string(),
            depends_on: vec!["dbus".to_string(), "27.0.0".to_string()],
            restart: false,
        };
        let text = config.to_yaml_string();
        let back = ServiceConfig::from_yaml_str(&text).unwrap();
        assert_eq!(back.name, config.name);
        assert_eq!(back.execute, config.execute);
        assert_eq!(back.service_type, config.service_type);
        assert_eq!(back.user, config.user);
        assert_eq!(back.depends_on, config.depends_on);
        assert_eq!(back.restart, config.restart);
    }

    #[test]
    fn shipped_service_files_all_parse() {
        // Every `.service` file in the repo must load, so a malformed
        // example can never ship.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("services");
        let mut count = 0;
        for entry in std::fs::read_dir(&dir).expect("services directory exists") {
            let path = entry.expect("readable dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("service") {
                continue;
            }
            let config = ServiceConfig::from_file(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert_eq!(path.file_stem().unwrap(), std::ffi::OsStr::new(&config.name));
            count += 1;
        }
        assert!(count > 0, "no .service files found in {dir:?}");
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
