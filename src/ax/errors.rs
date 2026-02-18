/// Errors from the macOS Accessibility API layer.
use thiserror::Error;

/// Raw AX error code from `AXUIElementCopyAttributeValue` and friends.
type RawAXError = i32;

/// Typed errors from the AX layer.
#[derive(Debug, Error)]
pub enum AXError {
    /// Accessibility permission not granted. User must enable in System Settings.
    #[error("Accessibility permission not granted")]
    NotTrusted,

    /// The AXUIElement reference is no longer valid (app quit, element disappeared).
    #[error("AX element is no longer valid")]
    InvalidElement,

    /// The requested attribute is not supported by this element.
    #[error("Attribute '{0}' not supported by this element")]
    AttributeUnsupported(String),

    /// The requested action is not supported by this element.
    #[error("Action '{0}' not supported by this element")]
    ActionUnsupported(String),

    /// The AX API call timed out (app not responding).
    #[error("AX API call timed out — app may be unresponsive")]
    Timeout,

    /// Generic AX API failure with the raw error code.
    #[error("AX API failure (code {code}): {context}")]
    ApiFailure {
        /// Raw `AXError` return code.
        code: RawAXError,
        /// Human-readable context for which call failed.
        context: String,
    },

    /// No running application matched the identifier provided.
    #[error("No running application matches '{identifier}'")]
    AppNotFound {
        /// The app name, PID string, or bundle ID that was searched.
        identifier: String,
    },
}

/// Map a raw `accessibility_sys` AX error code to our typed `AXError`.
///
/// # Errors
///
/// Returns `Err(AXError)` for any non-success code.
pub fn check_ax_error(code: i32, context: &str) -> Result<(), AXError> {
    use accessibility_sys::{
        kAXErrorActionUnsupported, kAXErrorAttributeUnsupported, kAXErrorCannotComplete,
        kAXErrorInvalidUIElement, kAXErrorSuccess,
    };

    if code == kAXErrorSuccess {
        return Ok(());
    }

    Err(match code {
        c if c == kAXErrorInvalidUIElement => AXError::InvalidElement,
        c if c == kAXErrorAttributeUnsupported => AXError::AttributeUnsupported(context.to_owned()),
        c if c == kAXErrorActionUnsupported => AXError::ActionUnsupported(context.to_owned()),
        // kAXErrorCannotComplete usually means the app is busy / timed out
        c if c == kAXErrorCannotComplete => AXError::Timeout,
        c => AXError::ApiFailure {
            code: c,
            context: context.to_owned(),
        },
    })
}
