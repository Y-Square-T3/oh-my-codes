use super::{DaemonAction, DaemonCommand};
use omc_api::client::OmcClient;
use omc_service::{create_service_manager, find_omcd_binary};

#[cfg(target_os = "windows")]
fn is_running_as_admin() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned_length = 0u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_length,
        );

        CloseHandle(token);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(target_os = "windows")]
fn quote_argument(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if !arg.contains([' ', '\t', '"', '\0']) {
        return arg.to_string();
    }
    let mut result = String::with_capacity(arg.len() + 2);
    result.push('"');
    let mut backslash_count = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslash_count += 1;
        } else if c == '"' {
            result.push_str(&"\\".repeat(backslash_count * 2 + 1));
            result.push('"');
            backslash_count = 0;
        } else {
            result.push_str(&"\\".repeat(backslash_count));
            result.push(c);
            backslash_count = 0;
        }
    }
    result.push_str(&"\\".repeat(backslash_count * 2));
    result.push('"');
    result
}

#[cfg(target_os = "windows")]
fn elevate_and_rerun() -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    if std::env::args().any(|a| a == "--elevated") {
        eprintln!("Error: elevation was attempted but admin status is still not detected.");
        eprintln!("This may indicate a bug in the admin detection logic.");
        std::process::exit(1);
    }

    let exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut elevated_args = vec!["--elevated".to_string()];
    elevated_args.extend(args);

    let args_str = elevated_args
        .iter()
        .map(|arg| quote_argument(arg))
        .collect::<Vec<_>>()
        .join(" ");

    let exe_wide: Vec<u16> = OsStr::new(&exe)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let args_wide: Vec<u16> = OsStr::new(&args_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let verb_wide: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    println!("Requesting administrator privileges...");

    unsafe {
        let result = ShellExecuteW(
            std::ptr::null_mut(),
            verb_wide.as_ptr(),
            exe_wide.as_ptr(),
            args_wide.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );

        if result as isize <= 32 {
            match result as isize {
                5 => {
                    eprintln!("Error: administrator access was denied.");
                    std::process::exit(1);
                }
                2 => {
                    eprintln!("Error: executable not found.");
                    std::process::exit(1);
                }
                3 => {
                    eprintln!("Error: path not found.");
                    std::process::exit(1);
                }
                code => {
                    eprintln!("Error: elevation failed (code: {}).", code);
                    std::process::exit(1);
                }
            }
        }
    }

    std::process::exit(0);
}

pub async fn run(client: &OmcClient, cmd: DaemonCommand) -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_service_manager();

    #[cfg(target_os = "windows")]
    {
        let needs_admin = matches!(
            cmd.action,
            DaemonAction::Install { .. }
                | DaemonAction::Uninstall
                | DaemonAction::Start
                | DaemonAction::Stop
        );

        if needs_admin && !is_running_as_admin() {
            elevate_and_rerun()?;
        }
    }

    match cmd.action {
        DaemonAction::Install { bin } => {
            let binary_path = match bin {
                Some(p) => p,
                None => find_omcd_binary().map_err(|e| e.to_string())?,
            };
            let config = omc_service::ServiceConfig {
                binary_path,
                data_dir: None,
                config: None,
            };
            manager.install(&config).map_err(|e| e.to_string())?;
            println!("Daemon installed successfully");

            let status = manager.status().map_err(|e| e.to_string())?;
            if !matches!(status, omc_service::ServiceStatus::Running { .. }) {
                match manager.start() {
                    Ok(()) => println!("Daemon started"),
                    Err(e) => eprintln!("Warning: failed to start daemon: {e}"),
                }
            }
        }
        DaemonAction::Uninstall => {
            manager.uninstall().map_err(|e| e.to_string())?;
            println!("Daemon uninstalled successfully");
        }
        DaemonAction::Start => {
            manager.start().map_err(|e| e.to_string())?;
            println!("Daemon started");
        }
        DaemonAction::Stop => {
            manager.stop().map_err(|e| e.to_string())?;
            println!("Daemon stopped");
        }
        DaemonAction::Status => {
            let status = manager.status().map_err(|e| e.to_string())?;
            match status {
                omc_service::ServiceStatus::Running { pid } => {
                    let version = client.health().await.map(|h| h.version).ok();
                    match (pid, version) {
                        (Some(p), Some(v)) => {
                            println!("Daemon is running (pid: {p}, version: {v})")
                        }
                        (Some(p), None) => println!("Daemon is running (pid: {p})"),
                        (None, Some(v)) => println!("Daemon is running (version: {v})"),
                        (None, None) => println!("Daemon is running"),
                    }
                }
                omc_service::ServiceStatus::Stopped => println!("Daemon is stopped"),
                omc_service::ServiceStatus::NotInstalled => {
                    println!("Daemon is not installed")
                }
                omc_service::ServiceStatus::Unknown(s) => println!("Daemon status: {s}"),
            }
        }
    }
    Ok(())
}
