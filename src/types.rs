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

/// Value of the `user` field that resolves to the owner of the active seat
/// session instead of a fixed account name.
pub const SESSION_USER: &str = "session";

/// Read-only protection applied to system directories before `exec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectSystem {
    /// Nothing is remounted; `/usr` and `/boot` stay writable.
    Off,
    /// `/usr` and `/boot` are remounted read-only.
    Full,
    /// `Full`, plus the whole tree except `/dev`, `/proc` and `/sys`.
    Strict,
}

impl Default for ProtectSystem {
    fn default() -> Self {
        ProtectSystem::Off
    }
}

impl std::fmt::Display for ProtectSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtectSystem::Off => write!(f, "off"),
            ProtectSystem::Full => write!(f, "full"),
            ProtectSystem::Strict => write!(f, "strict"),
        }
    }
}

impl ProtectSystem {
    /// Spelling used in `.service` files. Case-insensitive.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "off" | "no" | "false" => Some(ProtectSystem::Off),
            "full" | "yes" | "true" => Some(ProtectSystem::Full),
            "strict" => Some(ProtectSystem::Strict),
            _ => None,
        }
    }

    /// Spelling written by [`ServiceConfig::to_yaml_string`].
    pub fn as_config_str(&self) -> &'static str {
        match self {
            ProtectSystem::Off => "off",
            ProtectSystem::Full => "full",
            ProtectSystem::Strict => "strict",
        }
    }

    /// `true` when any directory is remounted read-only.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, ProtectSystem::Off)
    }
}

/// Capability names in kernel order, with their numeric value. The table is
/// the single source of truth for [`Capability`] conversions.
const CAPABILITY_NAMES: &[(&str, u8)] = &[
    ("CAP_CHOWN", 0),
    ("CAP_DAC_OVERRIDE", 1),
    ("CAP_DAC_READ_SEARCH", 2),
    ("CAP_FOWNER", 3),
    ("CAP_FSETID", 4),
    ("CAP_KILL", 5),
    ("CAP_SETGID", 6),
    ("CAP_SETUID", 7),
    ("CAP_SETPCAP", 8),
    ("CAP_LINUX_IMMUTABLE", 9),
    ("CAP_NET_BIND_SERVICE", 10),
    ("CAP_NET_BROADCAST", 11),
    ("CAP_NET_ADMIN", 12),
    ("CAP_NET_RAW", 13),
    ("CAP_IPC_LOCK", 14),
    ("CAP_IPC_OWNER", 15),
    ("CAP_SYS_MODULE", 16),
    ("CAP_SYS_RAWIO", 17),
    ("CAP_SYS_CHROOT", 18),
    ("CAP_SYS_PTRACE", 19),
    ("CAP_SYS_PACCT", 20),
    ("CAP_SYS_ADMIN", 21),
    ("CAP_SYS_BOOT", 22),
    ("CAP_SYS_NICE", 23),
    ("CAP_SYS_RESOURCE", 24),
    ("CAP_SYS_TIME", 25),
    ("CAP_SYS_TTY_CONFIG", 26),
    ("CAP_MKNOD", 27),
    ("CAP_LEASE", 28),
    ("CAP_AUDIT_WRITE", 29),
    ("CAP_AUDIT_CONTROL", 30),
    ("CAP_SETFCAP", 31),
    ("CAP_MAC_OVERRIDE", 32),
    ("CAP_MAC_ADMIN", 33),
    ("CAP_SYSLOG", 34),
    ("CAP_WAKE_ALARM", 35),
    ("CAP_BLOCK_SUSPEND", 36),
    ("CAP_AUDIT_READ", 37),
    ("CAP_PERFMON", 38),
    ("CAP_BPF", 39),
    ("CAP_CHECKPOINT_RESTORE", 40),
];

/// One Linux capability a service may keep after the bounding set is
/// dropped. A typo in a `.service` file is rejected at load time instead of
/// silently granting the wrong privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability(u8);

impl Capability {
    pub fn from_config_str(s: &str) -> Option<Self> {
        CAPABILITY_NAMES
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(s))
            .map(|(_, value)| Capability(*value))
    }

    pub fn as_config_str(&self) -> &'static str {
        CAPABILITY_NAMES
            .iter()
            .find(|(_, value)| *value == self.0)
            .map(|(name, _)| *name)
            .unwrap_or("CAP_UNKNOWN")
    }

    /// Numeric capability value for `capset` and `PR_CAPBSET_DROP`.
    pub fn as_raw(&self) -> u8 {
        self.0
    }

    /// `Err` when `value` is not a capability this table knows.
    pub fn from_raw(value: u8) -> Result<Self, String> {
        CAPABILITY_NAMES
            .iter()
            .find(|(_, known)| *known == value)
            .map(|_| Capability(value))
            .ok_or_else(|| format!("unknown capability value: {value}"))
    }

    /// Every known capability, in kernel order.
    pub fn all() -> Vec<Capability> {
        CAPABILITY_NAMES
            .iter()
            .map(|(_, value)| Capability(*value))
            .collect()
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_config_str())
    }
}

/// Access bits of a [`DeviceRule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DevicePerms {
    pub read: bool,
    pub write: bool,
    pub mknod: bool,
}

impl DevicePerms {
    /// Parse the `rwm` permission group. A missing group means read and
    /// write, matching the `chmod` convention.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.is_empty() {
            return Ok(DevicePerms {
                read: true,
                write: true,
                mknod: false,
            });
        }
        let mut perms = DevicePerms::default();
        for c in text.chars() {
            match c {
                'r' => perms.read = true,
                'w' => perms.write = true,
                'm' => perms.mknod = true,
                other => return Err(format!("unknown device permission `{}`", other)),
            }
        }
        if !perms.read && !perms.write && !perms.mknod {
            return Err("device permission group is empty".to_string());
        }
        Ok(perms)
    }

    /// `true` when no access at all is granted.
    pub fn is_empty(&self) -> bool {
        !self.read && !self.write && !self.mknod
    }
}

/// Which devices a [`DeviceRule`] matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceSelector {
    /// `char 189` / `char-major:189` and the `block` variants. A major of
    /// `0` matches every device of that kind.
    Major { block: bool, major: u32 },
    /// `char-*` or `block-*`, every device of that kind.
    Any { block: bool },
    /// Absolute device path; `*` is a glob wildcard.
    Path(String),
}

/// One entry of the `device_allow` list. The daemon turns the whole list
/// into a cgroup v2 device filter program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRule {
    pub selector: DeviceSelector,
    pub perms: DevicePerms,
}

impl DeviceRule {
    /// Parse `<selector> <perms>`, for example `char 189 rwm` or
    /// `/dev/snd/* rw`. The permission group is optional and defaults to
    /// read and write.
    ///
    /// ```rust
    /// use launchpad_lib::types::{DeviceRule, DeviceSelector};
    ///
    /// let rule = DeviceRule::parse("char 189 rwm").unwrap();
    /// assert_eq!(rule.selector, DeviceSelector::Major { block: false, major: 189 });
    /// assert!(rule.perms.mknod);
    ///
    /// let rule = DeviceRule::parse("/dev/snd/* rw").unwrap();
    /// assert_eq!(rule.selector, DeviceSelector::Path("/dev/snd/*".to_string()));
    /// assert!(!rule.perms.mknod);
    /// ```
    ///
    /// Returns `Err` on a malformed selector, a relative path, an unknown
    /// permission character or trailing words.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parts = text.split_whitespace();
        let head = parts
            .next()
            .ok_or_else(|| "device rule is empty".to_string())?;
        let selector = Self::parse_selector(head, &mut parts)?;
        let perms = match parts.next() {
            Some(group) => DevicePerms::parse(group)?,
            None => DevicePerms {
                read: true,
                write: true,
                mknod: false,
            },
        };
        if parts.next().is_some() {
            return Err(format!("device rule `{}` has trailing words", text.trim()));
        }
        Ok(DeviceRule { selector, perms })
    }

    fn parse_selector(
        head: &str,
        rest: &mut std::str::SplitWhitespace<'_>,
    ) -> Result<DeviceSelector, String> {
        let lower = head.to_ascii_lowercase();

        for (prefix, block) in [("char-major:", false), ("block-major:", true)] {
            if let Some(major) = lower.strip_prefix(prefix) {
                return Ok(DeviceSelector::Major {
                    block,
                    major: parse_device_major(Some(major))?,
                });
            }
        }

        match lower.as_str() {
            "char" | "char-major" => Ok(DeviceSelector::Major {
                block: false,
                major: parse_device_major(rest.next())?,
            }),
            "block" | "block-major" => Ok(DeviceSelector::Major {
                block: true,
                major: parse_device_major(rest.next())?,
            }),
            "char-*" => Ok(DeviceSelector::Any { block: false }),
            "block-*" => Ok(DeviceSelector::Any { block: true }),
            _ if head.starts_with('/') => Ok(DeviceSelector::Path(head.to_string())),
            _ => Err(format!(
                "device selector `{}` must be a major spec or an absolute path",
                head
            )),
        }
    }

    /// Render the rule back to its `.service` spelling. Round-trips through
    /// [`DeviceRule::parse`].
    pub fn to_config_string(&self) -> String {
        let selector = match &self.selector {
            DeviceSelector::Major { block, major } => {
                format!("{} {}", if *block { "block" } else { "char" }, major)
            }
            DeviceSelector::Any { block } => {
                format!("{}-*", if *block { "block" } else { "char" })
            }
            DeviceSelector::Path(path) => path.clone(),
        };
        let mut perms = String::new();
        if self.perms.read {
            perms.push('r');
        }
        if self.perms.write {
            perms.push('w');
        }
        if self.perms.mknod {
            perms.push('m');
        }
        format!("{} {}", selector, perms)
    }
}

fn parse_device_major(text: Option<&str>) -> Result<u32, String> {
    let text = text.ok_or_else(|| "device major is missing".to_string())?;
    if text == "*" {
        return Ok(0);
    }
    if text.is_empty() {
        return Err("device major is missing".to_string());
    }
    if text.contains('*') {
        return Err(format!("invalid device major `{}`", text));
    }
    text.parse::<u32>()
        .map_err(|_| format!("invalid device major `{}`", text))
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
    pub no_new_privs: bool,
    pub capabilities: Vec<Capability>,
    pub private_tmp: bool,
    pub protect_system: ProtectSystem,
    pub protect_home: bool,
    pub read_only_paths: Vec<String>,
    pub inaccessible_paths: Vec<String>,
    pub memory_max: Option<u64>,
    pub tasks_max: Option<u64>,
    pub cpu_quota: Option<f64>,
    pub device_allow: Vec<DeviceRule>,
    pub env_allow: Vec<String>,
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
            no_new_privs: false,
            capabilities: Vec::new(),
            private_tmp: false,
            protect_system: ProtectSystem::Off,
            protect_home: false,
            read_only_paths: Vec::new(),
            inaccessible_paths: Vec::new(),
            memory_max: None,
            tasks_max: None,
            cpu_quota: None,
            device_allow: Vec::new(),
            env_allow: Vec::new(),
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

fn opt_bool(doc: &JsonValue, key: &str, default: bool) -> Result<bool, String> {
    match field(doc, key) {
        None => Ok(default),
        Some(JsonValue::Bool(b)) => Ok(*b),
        Some(JsonValue::Str(s)) => match s.as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            other => Err(format!("field `{}` is not a bool: {}", key, other)),
        },
        Some(_) => Err(format!("field `{}` must be a bool", key)),
    }
}

fn opt_str_list(doc: &JsonValue, key: &str) -> Result<Vec<String>, String> {
    match field(doc, key) {
        None => Ok(Vec::new()),
        Some(JsonValue::Array(items)) => items
            .iter()
            .map(|item| match item {
                JsonValue::Str(s) => Ok(s.clone()),
                _ => Err(format!("field `{}` must be a list of strings", key)),
            })
            .collect(),
        Some(_) => Err(format!("field `{}` must be a list", key)),
    }
}

fn opt_u64(doc: &JsonValue, key: &str) -> Result<Option<u64>, String> {
    match field(doc, key) {
        None => Ok(None),
        Some(JsonValue::Integer(v)) => u64::try_from(*v)
            .map(Some)
            .map_err(|_| format!("field `{}` must not be negative: {}", key, v)),
        Some(_) => Err(format!("field `{}` must be an integer", key)),
    }
}

fn opt_f64(doc: &JsonValue, key: &str) -> Result<Option<f64>, String> {
    match field(doc, key) {
        None => Ok(None),
        Some(JsonValue::Integer(v)) => Ok(Some(*v as f64)),
        Some(JsonValue::Float(v)) => Ok(Some(*v)),
        Some(_) => Err(format!("field `{}` must be a number", key)),
    }
}

/// Accepts a plain byte count or a `K` / `M` / `G` suffix in either the short
/// (`512M`) or long (`512MiB`) spelling, so limits stay readable.
fn parse_byte_size(key: &str, value: u64, suffix: &str) -> Result<u64, String> {
    let multiplier = match suffix.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1u64,
        "k" | "kb" | "kib" => 1024,
        "m" | "mb" | "mib" => 1024 * 1024,
        "g" | "gb" | "gib" => 1024 * 1024 * 1024,
        _ => {
            return Err(format!(
                "field `{}` has an unknown size suffix: {}",
                key,
                suffix.trim()
            ))
        }
    };
    value
        .checked_mul(multiplier)
        .ok_or_else(|| format!("field `{}` overflows a byte count", key))
}

fn opt_byte_size(doc: &JsonValue, key: &str) -> Result<Option<u64>, String> {
    match field(doc, key) {
        None => Ok(None),
        Some(JsonValue::Integer(v)) => {
            let value = u64::try_from(*v)
                .map_err(|_| format!("field `{}` must not be negative: {}", key, v))?;
            Ok(Some(value))
        }
        Some(JsonValue::Str(s)) => {
            let split = s
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or_else(|| s.len());
            let (digits, suffix) = s.split_at(split);
            let value = digits
                .parse::<u64>()
                .map_err(|_| format!("field `{}` is not a byte size: {}", key, s))?;
            Ok(Some(parse_byte_size(key, value, suffix)?))
        }
        Some(_) => Err(format!("field `{}` must be a byte size", key)),
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
    /// - `user: session` is kept verbatim; the daemon resolves it to the
    ///   seat session owner, see [`ServiceConfig::is_session_user`].
    /// - Every hardening field is optional and defaults to "no sandboxing",
    ///   so an existing `.service` file keeps loading unchanged.
    /// - `capabilities` entries must be known `CAP_*` names, `device_allow`
    ///   entries must be parsable rules; a typo returns `Err` at load time.
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

        let depends_on = opt_str_list(&doc, "depends_on")?;
        let restart = opt_bool(&doc, "restart", true)?;

        let capabilities = opt_str_list(&doc, "capabilities")?
            .iter()
            .map(|name| {
                Capability::from_config_str(name)
                    .ok_or_else(|| format!("unknown capability `{}`", name))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let device_allow = opt_str_list(&doc, "device_allow")?
            .iter()
            .map(|rule| DeviceRule::parse(rule))
            .collect::<Result<Vec<_>, _>>()?;

        let protect_system = match field(&doc, "protect_system") {
            None => ProtectSystem::Off,
            Some(JsonValue::Str(s)) => ProtectSystem::from_config_str(s)
                .ok_or_else(|| format!("unknown protect_system `{}`", s))?,
            Some(_) => return Err("field `protect_system` must be a string".to_string()),
        };

        let config = Self {
            name: need_str(&doc, "name")?,
            execute: need_str(&doc, "execute")?,
            service_type,
            user: need_str(&doc, "user")?,
            depends_on,
            restart,
            no_new_privs: opt_bool(&doc, "no_new_privs", false)?,
            capabilities,
            private_tmp: opt_bool(&doc, "private_tmp", false)?,
            protect_system,
            protect_home: opt_bool(&doc, "protect_home", false)?,
            read_only_paths: opt_str_list(&doc, "read_only_paths")?,
            inaccessible_paths: opt_str_list(&doc, "inaccessible_paths")?,
            memory_max: opt_byte_size(&doc, "memory_max")?,
            tasks_max: opt_u64(&doc, "tasks_max")?,
            cpu_quota: opt_f64(&doc, "cpu_quota")?,
            device_allow,
            env_allow: opt_str_list(&doc, "env_allow")?,
        };
        if config.name.is_empty() {
            return Err("Service name is empty".to_string());
        }
        if config.execute.is_empty() {
            return Err("Service execute path is empty".to_string());
        }
        Ok(config)
    }

    /// `true` when `user` is the `session` placeholder, which the daemon
    /// resolves to the owner of the active seat session.
    pub fn is_session_user(&self) -> bool {
        self.user.eq_ignore_ascii_case(SESSION_USER)
    }

    /// `true` when the config asks for any sandboxing at all. The daemon
    /// uses this to decide whether a cgroup is needed.
    pub fn is_hardened(&self) -> bool {
        self.no_new_privs
            || self.private_tmp
            || self.protect_system.is_enabled()
            || self.protect_home
            || !self.capabilities.is_empty()
            || !self.read_only_paths.is_empty()
            || !self.inaccessible_paths.is_empty()
            || self.memory_max.is_some()
            || self.tasks_max.is_some()
            || self.cpu_quota.is_some()
            || !self.device_allow.is_empty()
    }

    /// Render the config back to `.service` YAML. Round-trips through
    /// [`ServiceConfig::from_yaml_str`].
    pub fn to_yaml_string(&self) -> String {
        let strs = |values: &Vec<String>| {
            JsonValue::Array(values.iter().map(|v| JsonValue::Str(v.clone())).collect())
        };
        let caps = JsonValue::Array(
            self.capabilities
                .iter()
                .map(|c| JsonValue::Str(c.as_config_str().to_string()))
                .collect(),
        );
        let devices = JsonValue::Array(
            self.device_allow
                .iter()
                .map(|rule| JsonValue::Str(rule.to_config_string()))
                .collect(),
        );
        let number = |value: Option<u64>| match value {
            Some(v) => JsonValue::Integer(v as i64),
            None => JsonValue::Null,
        };
        let quota = match self.cpu_quota {
            Some(q) => JsonValue::Float(q),
            None => JsonValue::Null,
        };

        let mut entries = vec![
            ("name".to_string(), JsonValue::Str(self.name.clone())),
            ("execute".to_string(), JsonValue::Str(self.execute.clone())),
            (
                "type".to_string(),
                JsonValue::Str(self.service_type.as_config_str().to_string()),
            ),
            ("user".to_string(), JsonValue::Str(self.user.clone())),
            ("depends_on".to_string(), strs(&self.depends_on)),
            ("restart".to_string(), JsonValue::Bool(self.restart)),
            ("no_new_privs".to_string(), JsonValue::Bool(self.no_new_privs)),
            ("capabilities".to_string(), caps),
            ("private_tmp".to_string(), JsonValue::Bool(self.private_tmp)),
            (
                "protect_system".to_string(),
                JsonValue::Str(self.protect_system.as_config_str().to_string()),
            ),
            ("protect_home".to_string(), JsonValue::Bool(self.protect_home)),
            ("read_only_paths".to_string(), strs(&self.read_only_paths)),
            (
                "inaccessible_paths".to_string(),
                strs(&self.inaccessible_paths),
            ),
            ("memory_max".to_string(), number(self.memory_max)),
            ("tasks_max".to_string(), number(self.tasks_max)),
            ("cpu_quota".to_string(), quota),
            ("device_allow".to_string(), devices),
            ("env_allow".to_string(), strs(&self.env_allow)),
        ];
        entries.retain(|(_, value)| !value.is_null());
        yaml::to_yaml(&JsonValue::Object(entries))
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
            ..ServiceConfig::default()
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
    fn hardening_roundtrip() {
        let config = ServiceConfig {
            name: "pipewire".to_string(),
            execute: "/usr/bin/pipewire".to_string(),
            service_type: ServiceType::Sys,
            user: SESSION_USER.to_string(),
            no_new_privs: true,
            capabilities: vec![
                Capability::from_config_str("CAP_DAC_READ_SEARCH").unwrap(),
                Capability::from_config_str("cap_sys_nice").unwrap(),
            ],
            private_tmp: true,
            protect_system: ProtectSystem::Full,
            protect_home: true,
            read_only_paths: vec!["/usr".to_string(), "/System".to_string()],
            inaccessible_paths: vec!["/root".to_string()],
            memory_max: Some(512 * 1024 * 1024),
            tasks_max: Some(256),
            cpu_quota: Some(200.0),
            device_allow: vec![
                DeviceRule::parse("char 189 rwm").unwrap(),
                DeviceRule::parse("/dev/snd/* rw").unwrap(),
                DeviceRule::parse("char-* r").unwrap(),
            ],
            env_allow: vec!["PATH".to_string(), "XDG_RUNTIME_DIR".to_string()],
            ..ServiceConfig::default()
        };

        let back = ServiceConfig::from_yaml_str(&config.to_yaml_string()).unwrap();

        assert_eq!(back.no_new_privs, true);
        assert_eq!(back.capabilities, config.capabilities);
        assert_eq!(back.private_tmp, true);
        assert_eq!(back.protect_system, ProtectSystem::Full);
        assert_eq!(back.protect_home, true);
        assert_eq!(back.read_only_paths, config.read_only_paths);
        assert_eq!(back.inaccessible_paths, config.inaccessible_paths);
        assert_eq!(back.memory_max, config.memory_max);
        assert_eq!(back.tasks_max, config.tasks_max);
        assert_eq!(back.cpu_quota, config.cpu_quota);
        assert_eq!(back.device_allow, config.device_allow);
        assert_eq!(back.env_allow, config.env_allow);
        assert!(back.is_session_user());
        assert!(back.is_hardened());
    }

    #[test]
    fn old_service_file_has_no_hardening() {
        let config = ServiceConfig::from_yaml_str(
            "name: dock\nexecute: /bin/dock\ntype: sys\nuser: root\ndepends_on:\n  - compositor\n",
        )
        .unwrap();
        assert!(!config.no_new_privs);
        assert!(!config.private_tmp);
        assert!(!config.protect_home);
        assert_eq!(config.protect_system, ProtectSystem::Off);
        assert!(config.capabilities.is_empty());
        assert!(config.device_allow.is_empty());
        assert_eq!(config.memory_max, None);
        assert!(!config.is_hardened());
        assert!(!config.is_session_user());
    }

    #[test]
    fn session_user_placeholder() {
        for spelling in ["session", "SESSION", "Session"] {
            let config = ServiceConfig::from_yaml_str(&format!(
                "name: pipewire\nexecute: /usr/bin/pipewire\ntype: sys\nuser: {spelling}\n"
            ))
            .unwrap();
            assert!(config.is_session_user(), "{spelling} should resolve at runtime");
        }
        let config = ServiceConfig::from_yaml_str(
            "name: dock\nexecute: /bin/dock\ntype: sys\nuser: liveuser\n",
        )
        .unwrap();
        assert!(!config.is_session_user());
    }

    #[test]
    fn capability_names_round_trip() {
        assert_eq!(Capability::all().len(), 41);
        for capability in Capability::all() {
            let name = capability.as_config_str();
            assert_eq!(Capability::from_config_str(name), Some(capability));
            assert_eq!(Capability::from_raw(capability.as_raw()), Ok(capability));
        }
        assert_eq!(
            Capability::from_config_str("CAP_NET_ADMIN").map(|c| c.as_raw()),
            Some(12)
        );
        assert_eq!(Capability::from_config_str("CAP_NOPE"), None);
        assert!(Capability::from_raw(200).is_err());
    }

    #[test]
    fn device_rule_parsing() {
        assert_eq!(
            DeviceRule::parse("char 189 rwm").unwrap().selector,
            DeviceSelector::Major {
                block: false,
                major: 189
            }
        );
        assert_eq!(
            DeviceRule::parse("block-major:8 r").unwrap().selector,
            DeviceSelector::Major {
                block: true,
                major: 8
            }
        );
        assert_eq!(
            DeviceRule::parse("char-* rw").unwrap().selector,
            DeviceSelector::Any { block: false }
        );
        assert_eq!(
            DeviceRule::parse("/dev/snd/* rw").unwrap().selector,
            DeviceSelector::Path("/dev/snd/*".to_string())
        );

        let default_perms = DeviceRule::parse("/dev/null").unwrap().perms;
        assert!(default_perms.read && default_perms.write && !default_perms.mknod);

        for bad in [
            "",
            "   ",
            "char",
            "char abc r",
            "sound r",
            "/dev/snd/* rw extra",
            "/dev/snd/* rx",
        ] {
            assert!(DeviceRule::parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn byte_size_suffixes() {
        let parse = |value: &str| {
            ServiceConfig::from_yaml_str(&format!(
                "name: a\nexecute: /b\ntype: sys\nuser: root\nmemory_max: {value}\n"
            ))
            .map(|c| c.memory_max)
        };
        assert_eq!(parse("1048576"), Ok(Some(1048576)));
        assert_eq!(parse("1M"), Ok(Some(1024 * 1024)));
        assert_eq!(parse("512MiB"), Ok(Some(512 * 1024 * 1024)));
        assert_eq!(parse("2G"), Ok(Some(2 * 1024 * 1024 * 1024)));
        assert_eq!(parse("1T"), Err("field `memory_max` has an unknown size suffix: T".to_string()));
        assert_eq!(parse("-1"), Err("field `memory_max` must not be negative: -1".to_string()));
        assert_eq!(parse("lots"), Err("field `memory_max` is not a byte size: lots".to_string()));
    }

    #[test]
    fn hardening_typos_are_rejected_at_load_time() {
        for bad in [
            "capabilities:\n  - CAP_MADE_UP",
            "capabilities: notalist",
            "device_allow:\n  - sound rw",
            "device_allow:\n  - /dev/snd/* zzz",
            "protect_system: sometimes",
            "no_new_privs: perhaps",
            "tasks_max: -4",
            "cpu_quota: half",
            "read_only_paths: nope",
        ] {
            let text = format!("name: a\nexecute: /b\ntype: sys\nuser: root\n{bad}\n");
            assert!(
                ServiceConfig::from_yaml_str(&text).is_err(),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn protect_system_spellings() {
        assert_eq!(ProtectSystem::from_config_str("full"), Some(ProtectSystem::Full));
        assert_eq!(ProtectSystem::from_config_str("STRICT"), Some(ProtectSystem::Strict));
        assert_eq!(ProtectSystem::from_config_str("no"), Some(ProtectSystem::Off));
        assert_eq!(ProtectSystem::from_config_str("maybe"), None);
        assert!(!ProtectSystem::Off.is_enabled());
        assert!(ProtectSystem::Strict.is_enabled());
        assert_eq!(ProtectSystem::default(), ProtectSystem::Off);
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
