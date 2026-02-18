/// Accessibility permission check helpers.
use accessibility_sys::AXIsProcessTrusted;

use super::errors::AXError;

/// Check whether this process is trusted for Accessibility access.
///
/// Returns `Ok(())` if trusted, `Err(AXError::NotTrusted)` otherwise.
///
/// # Errors
///
/// Returns `Err(AXError::NotTrusted)` if Accessibility permission has not been granted.
pub fn ensure_trusted() -> Result<(), AXError> {
    // SAFETY: Safe C FFI call with no arguments. Returns a Boolean.
    let trusted = unsafe { AXIsProcessTrusted() };
    if trusted {
        Ok(())
    } else {
        Err(AXError::NotTrusted)
    }
}

/// Human-readable instructions for granting Accessibility permission.
pub fn permission_instructions() -> &'static str {
    "To grant Accessibility permission:\n  \
     1. Open System Settings → Privacy & Security → Accessibility\n  \
     2. Click the + button and add your terminal application\n  \
     3. Restart your terminal\n\n  \
     Or run: open \"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility\""
}
