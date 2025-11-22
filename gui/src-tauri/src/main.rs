// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Clone, Serialize)]
struct InstallProgress {
    step: String,
    status: String, // "info", "ok", "warn", "error"
    message: String,
    progress: u8, // 0-100
}

#[derive(Deserialize)]
struct InstallOptions {
    method: String, // "Auto", "Mcpb", "Dxt", "Manual"
}

#[tauri::command]
async fn run_installer(
    options: InstallOptions,
    window: tauri::Window,
) -> Result<String, String> {
    // Get the installer script URL
    let script_url = "https://raw.githubusercontent.com/Stryk91/PhiSHRI/main/install.ps1";

    // Build PowerShell command
    let ps_command = format!(
        "$ErrorActionPreference = 'Continue'; \
         $script = Invoke-RestMethod -Uri '{}'; \
         $scriptBlock = [ScriptBlock]::Create($script); \
         & $scriptBlock -Method {}",
        script_url,
        options.method
    );

    // Emit starting event
    let _ = window.emit("install-progress", InstallProgress {
        step: "Starting".to_string(),
        status: "info".to_string(),
        message: format!("Starting installation with method: {}", options.method),
        progress: 0,
    });

    // Run PowerShell
    let mut child = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command", &ps_command,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start PowerShell: {}", e))?;

    let stdout = child.stdout.take()
        .ok_or("Failed to capture stdout")?;

    let mut reader = BufReader::new(stdout).lines();
    let mut progress: u8 = 10;
    let mut output_lines: Vec<String> = Vec::new();

    while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
        output_lines.push(line.clone());

        // Parse output and emit progress events
        let (status, step) = parse_installer_output(&line);

        // Increment progress based on detected steps
        if line.contains("[OK]") || line.contains("OK") {
            progress = (progress + 10).min(95);
        }

        let _ = window.emit("install-progress", InstallProgress {
            step: step.clone(),
            status,
            message: line.clone(),
            progress,
        });
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;

    if status.success() {
        let _ = window.emit("install-progress", InstallProgress {
            step: "Complete".to_string(),
            status: "ok".to_string(),
            message: "Installation completed successfully!".to_string(),
            progress: 100,
        });
        Ok("Installation completed successfully".to_string())
    } else {
        let _ = window.emit("install-progress", InstallProgress {
            step: "Failed".to_string(),
            status: "error".to_string(),
            message: "Installation failed. Check logs for details.".to_string(),
            progress: 100,
        });
        Err(format!("Installation failed with exit code: {:?}", status.code()))
    }
}

fn parse_installer_output(line: &str) -> (String, String) {
    if line.contains("[OK]") {
        ("ok".to_string(), extract_step(line))
    } else if line.contains("[WARN]") {
        ("warn".to_string(), extract_step(line))
    } else if line.contains("[ERROR]") {
        ("error".to_string(), extract_step(line))
    } else if line.contains("[INFO]") {
        ("info".to_string(), extract_step(line))
    } else {
        ("info".to_string(), "Processing".to_string())
    }
}

fn extract_step(line: &str) -> String {
    if line.contains("prerequisites") || line.contains("Prerequisites") {
        "Checking prerequisites".to_string()
    } else if line.contains("directory") || line.contains("Directory") {
        "Creating directories".to_string()
    } else if line.contains("binary") || line.contains("Binary") || line.contains("MCP") {
        "Downloading MCP binary".to_string()
    } else if line.contains("knowledge") || line.contains("Knowledge") || line.contains("CONTEXTS") {
        "Installing knowledge base".to_string()
    } else if line.contains("config") || line.contains("Config") || line.contains("Claude") {
        "Configuring Claude Desktop".to_string()
    } else if line.contains("verif") || line.contains("Verif") {
        "Verifying installation".to_string()
    } else {
        "Processing".to_string()
    }
}

#[tauri::command]
async fn run_uninstaller(window: tauri::Window) -> Result<String, String> {
    let script_url = "https://raw.githubusercontent.com/Stryk91/PhiSHRI/main/uninstall.ps1";

    let ps_command = format!(
        "Invoke-RestMethod -Uri '{}' | Invoke-Expression",
        script_url
    );

    let _ = window.emit("install-progress", InstallProgress {
        step: "Uninstalling".to_string(),
        status: "info".to_string(),
        message: "Removing PhiSHRI...".to_string(),
        progress: 50,
    });

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command", &ps_command,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run uninstaller: {}", e))?;

    if output.status.success() {
        let _ = window.emit("install-progress", InstallProgress {
            step: "Complete".to_string(),
            status: "ok".to_string(),
            message: "PhiSHRI has been uninstalled".to_string(),
            progress: 100,
        });
        Ok("Uninstall completed".to_string())
    } else {
        Err("Uninstall failed".to_string())
    }
}

#[tauri::command]
async fn check_installation() -> Result<serde_json::Value, String> {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    let phishri_root = format!("{}/.phishri", home);
    let binary_path = format!("{}/bin/phishri-mcp.exe", phishri_root);
    let knowledge_path = format!("{}/knowledge/CONTEXTS", phishri_root);

    let binary_exists = std::path::Path::new(&binary_path).exists();
    let knowledge_exists = std::path::Path::new(&knowledge_path).exists();

    // Count doors if knowledge exists
    let door_count = if knowledge_exists {
        count_json_files(&knowledge_path).unwrap_or(0)
    } else {
        0
    };

    Ok(serde_json::json!({
        "installed": binary_exists && knowledge_exists,
        "binary_exists": binary_exists,
        "knowledge_exists": knowledge_exists,
        "door_count": door_count,
        "paths": {
            "root": phishri_root,
            "binary": binary_path,
            "knowledge": knowledge_path
        }
    }))
}

fn count_json_files(path: &str) -> Result<usize, std::io::Error> {
    let mut count = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            count += count_json_files(path.to_str().unwrap_or(""))?;
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            count += 1;
        }
    }
    Ok(count)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_installer,
            run_uninstaller,
            check_installation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
