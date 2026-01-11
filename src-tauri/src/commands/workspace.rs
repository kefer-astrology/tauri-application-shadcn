use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::workspace::{load_workspace_manifest, load_all_charts, chart_to_summary, ChartSummary, WorkspaceInfo};

/// Open a folder dialog and return the selected path
#[tauri::command]
pub async fn open_folder_dialog() -> Result<Option<String>, String> {
    // Use native file dialog via system command
    // This is a simple cross-platform approach
    #[cfg(target_os = "windows")]
    {
        // Windows: use PowerShell
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; if ($dialog.ShowDialog() -eq 'OK') { $dialog.SelectedPath }"
            ])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(path))
                }
            }
            _ => Ok(None)
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: use osascript
        let script = r#"tell application "System Events"
    activate
    set folderPath to choose folder with prompt "Select Workspace Folder"
    return POSIX path of folderPath
end tell"#;
        
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(path))
                }
            }
            _ => Ok(None)
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux: try zenity, kdialog, or yad
        let commands = vec![
            ("zenity", vec!["--file-selection", "--directory", "--title=Select Workspace Folder"]),
            ("kdialog", vec!["--getexistingdirectory", ".", "--title", "Select Workspace Folder"]),
            ("yad", vec!["--file", "--directory", "--title=Select Workspace Folder"]),
        ];
        
        for (cmd, args) in commands {
            if let Ok(output) = Command::new(cmd).args(args).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(Some(path));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}


/// Load workspace from a directory containing workspace.yaml
#[tauri::command]
pub async fn load_workspace(workspace_path: String) -> Result<WorkspaceInfo, String> {
    let workspace_dir = Path::new(&workspace_path);
    
    // Load manifest using Rust YAML parser
    let manifest = load_workspace_manifest(workspace_dir)?;
    
    // Load all charts
    let charts = load_all_charts(workspace_dir, &manifest)?;
    
    // Convert to summaries
    let chart_summaries: Vec<ChartSummary> = charts.iter()
        .map(chart_to_summary)
        .collect();
    
    Ok(WorkspaceInfo {
        path: workspace_path,
        owner: manifest.owner,
        active_model: manifest.active_model,
        charts: chart_summaries,
    })
}

/// Get full chart details including all settings
#[tauri::command]
pub async fn get_chart_details(
    workspace_path: String,
    chart_id: String,
) -> Result<serde_json::Value, String> {
    use crate::workspace::loader::load_chart;
    use serde_json::json;
    
    let workspace_dir = Path::new(&workspace_path);
    
    // Find chart file in manifest
    let manifest = load_workspace_manifest(workspace_dir)?;
    let chart_path = manifest.charts.iter()
        .find(|p| p.contains(&chart_id))
        .ok_or_else(|| format!("Chart {} not found in manifest", chart_id))?;
    
    let chart = load_chart(workspace_dir, chart_path)?;
    
    // Serialize to JSON
    
    let mode_str = match chart.config.mode {
        crate::workspace::models::ChartMode::NATAL => "NATAL",
        crate::workspace::models::ChartMode::EVENT => "EVENT",
        crate::workspace::models::ChartMode::HORARY => "HORARY",
        crate::workspace::models::ChartMode::COMPOSITE => "COMPOSITE",
    };
    
    let house_system_str = chart.config.house_system.as_ref().map(|h| {
        match h {
            crate::workspace::models::HouseSystem::Placidus => "Placidus",
            crate::workspace::models::HouseSystem::WholeSign => "Whole Sign",
            crate::workspace::models::HouseSystem::Campanus => "Campanus",
            crate::workspace::models::HouseSystem::Koch => "Koch",
            crate::workspace::models::HouseSystem::Equal => "Equal",
            crate::workspace::models::HouseSystem::Regiomontanus => "Regiomontanus",
            crate::workspace::models::HouseSystem::Vehlow => "Vehlow",
            crate::workspace::models::HouseSystem::Porphyry => "Porphyry",
            crate::workspace::models::HouseSystem::Alcabitius => "Alcabitius",
        }
    });
    
    let zodiac_type_str = match chart.config.zodiac_type {
        crate::workspace::models::ZodiacType::Tropical => "Tropical",
        crate::workspace::models::ZodiacType::Sidereal => "Sidereal",
    };
    
    let engine_str = chart.config.engine.as_ref().map(|e| {
        match e {
            crate::workspace::models::EngineType::Swisseph => "swisseph",
            crate::workspace::models::EngineType::Jyotish => "jyotish",
            crate::workspace::models::EngineType::Jpl => "jpl",
            crate::workspace::models::EngineType::Custom => "custom",
        }
    });
    
    Ok(json!({
        "id": chart.id,
        "subject": {
            "id": chart.subject.id,
            "name": chart.subject.name,
            "event_time": chart.subject.event_time.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
            "location": {
                "name": chart.subject.location.name,
                "latitude": chart.subject.location.latitude,
                "longitude": chart.subject.location.longitude,
                "timezone": chart.subject.location.timezone,
            }
        },
        "config": {
            "mode": mode_str,
            "house_system": house_system_str,
            "zodiac_type": zodiac_type_str,
            "engine": engine_str,
            "model": chart.config.model,
            "override_ephemeris": chart.config.override_ephemeris,
        },
        "tags": chart.tags,
    }))
}

/// Compute chart positions and aspects using Python
#[tauri::command]
pub async fn compute_chart(
    workspace_path: String,
    chart_id: String,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let python_exe = find_python_executable()?;
    let module_path = get_module_path()?;
    
    let output = Command::new(&python_exe)
        .arg("-c")
        .arg(format!(
            r#"
import sys
import json
from pathlib import Path
sys.path.insert(0, '{}')
from module.workspace import load_workspace
from module.services import compute_positions_for_chart

workspace_path = r'{}'
chart_id = r'{}'

ws = load_workspace(workspace_path)
chart = next((ch for ch in (ws.charts or []) if getattr(ch, 'id', None) == chart_id), None)

if not chart:
    print(json.dumps({{'error': 'Chart not found'}}))
    sys.exit(1)

try:
    positions = compute_positions_for_chart(chart)
    aspects = []  # TODO: compute aspects
    
    result = {{
        'positions': positions or {{}},
        'aspects': aspects,
        'chart_id': chart_id
    }}
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({{'error': str(e)}}))
    sys.exit(1)
"#,
            module_path.display(),
            workspace_path,
            chart_id
        ))
        .current_dir(&module_path)
        .output()
        .map_err(|e| format!("Failed to execute Python: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python error: {}", error));
    }
    
    let result_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to read Python output: {}", e))?;
    
    let result: HashMap<String, serde_json::Value> = serde_json::from_str(&result_str)
        .map_err(|e| format!("Failed to parse computation result: {}", e))?;
    
    if let Some(error) = result.get("error") {
        return Err(error.as_str().unwrap().to_string());
    }
    
    Ok(result)
}

fn find_python_executable() -> Result<PathBuf, String> {
    let candidates = vec!["python3", "python", "py"];
    
    for cmd in candidates {
        if Command::new(cmd)
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(PathBuf::from(cmd));
        }
    }
    
    Err("Python executable not found".to_string())
}

fn get_module_path() -> Result<PathBuf, String> {
    // Get the external_package directory relative to the workspace root
    // In development, this should be the external_package directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Try to find external_package directory
    // First try current directory
    let candidates = vec![
        current_dir.join("external_package"),
    ];
    
    // Then try parent directory
    if let Some(parent) = current_dir.parent() {
        let parent_candidates = vec![
            parent.join("external_package"),
        ];
        for path in parent_candidates {
            if path.exists() && path.is_dir() {
                return Ok(path);
            }
        }
    }
    
    // Try canonicalized path
    if let Ok(canonical) = current_dir.join("..").join("external_package").canonicalize() {
        if canonical.exists() && canonical.is_dir() {
            return Ok(canonical);
        }
    }
    
    for path in candidates {
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }
    
    Err("external_package directory not found".to_string())
}
