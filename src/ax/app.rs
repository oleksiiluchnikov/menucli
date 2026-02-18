/// App PID resolution via NSWorkspace.
use objc2_app_kit::NSWorkspace;

use super::errors::AXError;

/// Resolve an app identifier string (name, bundle ID, or PID integer) to a PID.
///
/// Resolution order:
/// 1. If the string is a valid integer → treat as PID directly.
/// 2. If the string contains a `.` → treat as bundle ID (exact match).
/// 3. Otherwise → treat as app name (case-insensitive substring match).
///
/// # Errors
///
/// Returns `Err(AXError::AppNotFound)` if no running application matches.
pub fn resolve_app_pid(identifier: &str) -> Result<i32, AXError> {
    // 1. Direct PID
    if let Ok(pid) = identifier.parse::<i32>() {
        return Ok(pid);
    }

    let is_bundle_id = identifier.contains('.');

    let workspace = NSWorkspace::sharedWorkspace();
    let apps = workspace.runningApplications();

    for app in apps.iter() {
        if is_bundle_id {
            if let Some(bid) = app.bundleIdentifier() {
                if bid.to_string() == identifier {
                    return Ok(app.processIdentifier());
                }
            }
        } else {
            // Name match: case-insensitive contains
            if let Some(name) = app.localizedName() {
                if name
                    .to_string()
                    .to_lowercase()
                    .contains(&identifier.to_lowercase())
                {
                    return Ok(app.processIdentifier());
                }
            }
        }
    }

    Err(AXError::AppNotFound {
        identifier: identifier.to_owned(),
    })
}

/// Get the PID of the frontmost (focused) application.
///
/// # Errors
///
/// Returns `Err(AXError::AppNotFound)` if no frontmost application can be determined.
pub fn frontmost_app_pid() -> Result<i32, AXError> {
    let workspace = NSWorkspace::sharedWorkspace();
    if let Some(app) = workspace.frontmostApplication() {
        return Ok(app.processIdentifier());
    }
    Err(AXError::AppNotFound {
        identifier: "<frontmost>".to_owned(),
    })
}

/// Get info for all running applications.
pub struct RunningApp {
    pub name: String,
    pub pid: i32,
    pub bundle_id: Option<String>,
    pub frontmost: bool,
}

/// List all running applications with GUI access.
pub fn list_running_apps() -> Vec<RunningApp> {
    let mut result = Vec::new();
    let workspace = NSWorkspace::sharedWorkspace();
    let apps = workspace.runningApplications();
    let frontmost_pid = workspace
        .frontmostApplication()
        .map(|a| a.processIdentifier());

    for app in apps.iter() {
        let pid = app.processIdentifier();
        let name = app
            .localizedName()
            .map(|n| n.to_string())
            .unwrap_or_default();
        let bundle_id = app.bundleIdentifier().map(|b| b.to_string());
        let frontmost = frontmost_pid == Some(pid);
        result.push(RunningApp {
            name,
            pid,
            bundle_id,
            frontmost,
        });
    }
    // Filter to only apps with a name (background agents have empty names)
    result.retain(|a| !a.name.is_empty());
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Resolve an optional `--app` flag to a PID.
/// If `None`, returns the frontmost app PID.
///
/// # Errors
///
/// Returns `Err(AXError::AppNotFound)` if the app cannot be resolved.
pub fn resolve_target(app: Option<&str>) -> Result<i32, AXError> {
    match app {
        Some(identifier) => resolve_app_pid(identifier),
        None => frontmost_app_pid(),
    }
}
