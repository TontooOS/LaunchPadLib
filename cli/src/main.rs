#[cfg(target_os = "linux")]
mod platform {
    use clap::{Parser, Subcommand};
    use launchpad_lib::client::LaunchpadClient;

    #[derive(Parser)]
    #[command(name = "launchctl")]
    #[command(about = "LaunchPad service manager for TontooOS")]
    #[command(version = "0.1.0")]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        /// Start a service
        Start {
            /// Service name
            name: String,
        },
        /// Stop a service (graceful: SIGTERM -> 5s -> SIGKILL)
        Stop {
            /// Service name
            name: String,
        },
        /// Restart a service
        Restart {
            /// Service name
            name: String,
        },
        /// Kill a service immediately (SIGKILL)
        Kill {
            /// Service name
            name: String,
        },
        /// Enable autostart for a service
        Activate {
            /// Service name
            name: String,
        },
        /// Disable autostart for a service
        Deactivate {
            /// Service name
            name: String,
        },
        /// List all services with status
        List,
        /// Show service log
        Log {
            /// Service name
            name: String,
            /// Show only last N lines
            #[arg(short, long)]
            head: Option<usize>,
        },
        /// Start an .app bundle
        StartApp {
            /// Path to .app bundle
            path: String,
        },
    }

    pub fn main() {
        let cli = Cli::parse();

        let client = match LaunchpadClient::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        let result = match &cli.command {
            Commands::Start { name } => client.start(name).map(|_| ()),
            Commands::Stop { name } => client.stop(name).map(|_| ()),
            Commands::Restart { name } => client.restart(name).map(|_| ()),
            Commands::Kill { name } => client.kill(name).map(|_| ()),
            Commands::Activate { name } => client.activate(name).map(|_| ()),
            Commands::Deactivate { name } => client.deactivate(name).map(|_| ()),
            Commands::List => client.list().map(|services| {
                println!(
                    "{:<30} {:<10} {:<12} {:<10} {:<10}",
                    "NAME", "TYPE", "STATE", "PID", "USER"
                );
                println!("{}", "-".repeat(72));
                for s in &services {
                    let pid = s
                        .pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{:<30} {:<10} {:<12} {:<10} {:<10}",
                        s.name, s.service_type, s.state, pid, s.user
                    );
                }
            }),
            Commands::Log { name, head } => client.log(name, *head).map(|response| {
                if let Some(msg) = &response.message {
                    println!("{}", msg);
                }
            }),
            Commands::StartApp { path } => client.start_app(path).map(|info| {
                println!(
                    "Started '{}' as '{}' (PID: {})",
                    path,
                    info.name,
                    info.pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string())
                );
            }),
        };

        if let Err(e) = result {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    platform::main();

    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("launchctl is only supported on Linux");
        std::process::exit(1);
    }
}
