#![cfg(target_os = "linux")]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

use launchpad_lib::types::{IpcData, IpcRequest, IpcResponse, SOCKET_PATH};

use crate::manager::ServiceManager;

pub fn start_socket_server(
    manager: Arc<Mutex<ServiceManager>>,
) -> Result<(), String> {
    let socket_path = Path::new(SOCKET_PATH);

    // Remove old socket
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .map_err(|e| format!("Cannot remove old socket: {}", e))?;
    }

    let listener = UnixListener::bind(socket_path)
        .map_err(|e| format!("Cannot bind socket: {}", e))?;

    // Make socket accessible
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(
            socket_path,
            std::fs::Permissions::from_mode(0o666),
        );
    }

    log::info!("Socket server listening on {}", SOCKET_PATH);

    listener.set_nonblocking(true).map_err(|e| format!("Cannot set nonblocking: {}", e))?;

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                handle_client(stream, &manager);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                log::warn!("Socket accept error: {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn handle_client(mut stream: UnixStream, manager: &Arc<Mutex<ServiceManager>>) {
    let reader_stream = stream.try_clone().ok();
    if let Some(reader) = reader_stream {
        let mut buf_reader = BufReader::new(reader);
        let mut request_line = String::new();

        if buf_reader.read_line(&mut request_line).is_err() || request_line.trim().is_empty() {
            return;
        }

        let request: IpcRequest = match serde_json::from_str(request_line.trim()) {
            Ok(r) => r,
            Err(e) => {
                let response = IpcResponse {
                    success: false,
                    message: Some(format!("Invalid request: {}", e)),
                    data: None,
                };
                send_response(&mut stream, &response);
                return;
            }
        };

        let response = process_request(request, manager);
        send_response(&mut stream, &response);
    }
}

fn process_request(request: IpcRequest, manager: &Arc<Mutex<ServiceManager>>) -> IpcResponse {
    let mut mgr = manager.lock().unwrap();

    match request.action.as_str() {
        "start" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.start(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "stop" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.stop(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "restart" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.restart(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "kill" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.kill(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "activate" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.activate(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "deactivate" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.deactivate(&name) {
                Ok(()) => success_response(None),
                Err(e) => error_response(&e),
            }
        }
        "list" => {
            let services = mgr.list();
            IpcResponse {
                success: true,
                message: None,
                data: Some(IpcData { services }),
            }
        }
        "log" => {
            let name = match &request.service {
                Some(n) => n.clone(),
                None => return error_response("Missing service name"),
            };
            match mgr.log_service(&name, request.options.head) {
                Ok(lines) => {
                    let msg = lines.join("\n");
                    IpcResponse {
                        success: true,
                        message: Some(msg),
                        data: None,
                    }
                }
                Err(e) => error_response(&e),
            }
        }
        "start_app" => {
            let app_path = match &request.options.app_path {
                Some(p) => p.clone(),
                None => return error_response("Missing app_path"),
            };
            match mgr.start_app_bundle(&app_path) {
                Ok(id) => {
                    let services = mgr.list();
                    let service = services.into_iter().find(|s| s.name == id);
                    IpcResponse {
                        success: true,
                        message: None,
                        data: service.map(|s| IpcData {
                            services: vec![s],
                        }),
                    }
                }
                Err(e) => error_response(&e),
            }
        }
        _ => error_response(&format!("Unknown action: {}", request.action)),
    }
}

fn send_response(stream: &mut UnixStream, response: &IpcResponse) {
    if let Ok(json) = serde_json::to_string(response) {
        let _ = stream.write_all(json.as_bytes());
        let _ = stream.write_all(b"\n");
    }
}

fn success_response(message: Option<&str>) -> IpcResponse {
    IpcResponse {
        success: true,
        message: message.map(String::from),
        data: None,
    }
}

fn error_response(msg: &str) -> IpcResponse {
    IpcResponse {
        success: false,
        message: Some(msg.to_string()),
        data: None,
    }
}
