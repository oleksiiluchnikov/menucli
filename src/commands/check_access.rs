/// `check-access` command: verify Accessibility permission is granted.
use crate::ax::{ensure_trusted, permission_instructions};
use crate::cli::OutputCtx;
use crate::menu::MenuError;

/// Run `menucli check-access`.
///
/// Exits 0 if trusted, exits 3 with an error message if not.
///
/// # Errors
///
/// Returns `MenuError::AccessDenied` if permission is not granted.
pub fn run(ctx: &OutputCtx) -> Result<(), MenuError> {
    ensure_trusted().map_err(|_| MenuError::AccessDenied)?;

    match ctx.format {
        crate::cli::OutputFormat::Json
        | crate::cli::OutputFormat::Compact
        | crate::cli::OutputFormat::Ndjson => {
            println!(r#"{{"ok":true,"message":"Accessibility permission granted"}}"#);
        }
        _ => {
            println!("Accessibility permission granted.");
            println!("{}", permission_instructions());
        }
    }

    Ok(())
}
