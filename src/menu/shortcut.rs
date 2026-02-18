/// Menu item keyboard shortcut formatting.
///
/// The AX API reports shortcuts as two attributes:
/// - `kAXMenuItemCmdChar`: The key character (e.g., "S", "N", "W").
/// - `kAXMenuItemCmdModifiers`: A bitmask of modifier keys.
///
/// Modifier bitmask (from Apple's `AXAttributeConstants.h`):
/// - 0 = ⌘ only (default — no bit set means Command is implied)
/// - bit 1 (0x1) = Shift (⇧)
/// - bit 2 (0x2) = Option (⌥)
/// - bit 3 (0x4) = Control (⌃)
/// - bit 4 (0x8) = No Command (⌘ is NOT pressed; modifier-only shortcut)
///
/// Note: This is NOT the standard Carbon/Cocoa modifier mask. The bit layout
/// is specific to the AX API's `kAXMenuItemCmdModifiers` attribute.
/// Reference: AXAttributeConstants.h kAXMenuItemModifier* constants.

/// Format a keyboard shortcut string from AX attribute values.
///
/// Returns `None` if there is no keyboard shortcut (empty `cmd_char`).
///
/// # Examples
///
/// ```
/// // "S" with modifiers 0 (Command only) → "⌘S"
/// // "S" with modifiers 1 (Shift+Command) → "⇧⌘S"
/// // "S" with modifiers 3 (Option+Command) → "⌥⌘S"
/// ```
#[must_use]
pub fn format_shortcut(cmd_char: Option<&str>, modifiers: Option<i64>) -> Option<String> {
    let char = cmd_char?.trim();
    if char.is_empty() {
        return None;
    }

    let mods = modifiers.unwrap_or(0);
    let mut shortcut = String::with_capacity(8);

    let has_shift = (mods & 0x1) != 0;
    let has_option = (mods & 0x2) != 0;
    let has_control = (mods & 0x4) != 0;
    let no_command = (mods & 0x8) != 0;

    if has_control {
        shortcut.push('⌃');
    }
    if has_option {
        shortcut.push('⌥');
    }
    if has_shift {
        shortcut.push('⇧');
    }
    if !no_command {
        shortcut.push('⌘');
    }

    shortcut.push_str(char);
    Some(shortcut)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_only() {
        assert_eq!(format_shortcut(Some("S"), Some(0)), Some("⌘S".to_owned()));
    }

    #[test]
    fn test_shift_command() {
        assert_eq!(format_shortcut(Some("S"), Some(1)), Some("⇧⌘S".to_owned()));
    }

    #[test]
    fn test_option_command() {
        assert_eq!(format_shortcut(Some("W"), Some(2)), Some("⌥⌘W".to_owned()));
    }

    #[test]
    fn test_control_only() {
        assert_eq!(
            format_shortcut(Some("F"), Some(0x4 | 0x8)),
            Some("⌃F".to_owned())
        );
    }

    #[test]
    fn test_no_char() {
        assert_eq!(format_shortcut(None, Some(0)), None);
        assert_eq!(format_shortcut(Some(""), Some(0)), None);
    }
}
