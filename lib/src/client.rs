use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::types::{IpcRequest, IpcResponse, ServiceInfo, SOCKET_PATH};

#[derive(Debug)]
pub struct LaunchpadClient {
    socket_path: String,
}

impl LaunchpadClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            socket_path: SOCKET_PATH.to_string(),
        })
    }

    pub fn with_socket(path: &str) -> Self {
        Self {
            socket_path: path.to_string(),
        }
    }

    fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Cannot connect to LaunchPad daemon: {}", e))?;

        let json =
            serde_json::to_string(request).map_err(|e| format!("Serialize error: {}", e))?;
        stream
            .write_all(json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        stream
            .write_all(b"\n")
            .map_err(|e| format!("Write error: {}", e))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("Read error: {}", e))?;

        serde_json::from_str(&response).map_err(|e| format!("Deserialize error: {}", e))
    }

    pub fn start(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "start".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn stop(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "stop".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn restart(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "restart".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn kill(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "kill".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn activate(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "activate".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn deactivate(&self, service: &str) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "deactivate".to_string(),
            service: Some(service.to_string()),
            options: Default::default(),
        })
    }

    pub fn list(&self) -> Result<Vec<ServiceInfo>, String> {
        let response = self.send_request(&IpcRequest {
            action: "list".to_string(),
            service: None,
            options: Default::default(),
        })?;
        Ok(response.data.map(|d| d.services).unwrap_or_default())
    }

    pub fn log(&self, service: &str, head: Option<usize>) -> Result<IpcResponse, String> {
        self.send_request(&IpcRequest {
            action: "log".to_string(),
            service: Some(service.to_string()),
            options: crate::types::IpcOptions {
                head,
                app_path: None,
            },
        })
    }

    pub fn start_app(&self, app_path: &str) -> Result<ServiceInfo, String> {
        let response = self.send_request(&IpcRequest {
            action: "start_app".to_string(),
            service: None,
            options: crate::types::IpcOptions {
                head: None,
                app_path: Some(app_path.to_string()),
            },
        })?;
        response
            .data
            .and_then(|d| d.services.into_iter().next())
            .ok_or_else(|| "No service info returned".to_string())
    }

    pub fn is_running(&self, app_name: &str) -> Result<bool, String> {
        let services = self.list()?;
        Ok(services
            .iter()
            .any(|s| s.name.contains(app_name) && s.state == crate::types::ServiceState::Running))
    }
}
