/// Safe wrapper around AXUIElementRef with batch attribute fetching.
use accessibility_sys::{
    kAXChildrenAttribute, kAXEnabledAttribute, kAXErrorSuccess, kAXExtrasMenuBarAttribute,
    kAXMenuBarAttribute, kAXMenuItemCmdCharAttribute, kAXMenuItemCmdModifiersAttribute,
    kAXMenuItemMarkCharAttribute, kAXMenuItemPrimaryUIElementAttribute, kAXRoleAttribute,
    kAXTitleAttribute, kAXVisibleChildrenAttribute, AXUIElementCopyAttributeValue,
    AXUIElementCopyMultipleAttributeValues, AXUIElementCreateApplication, AXUIElementGetPid,
    AXUIElementPerformAction, AXUIElementRef, AXUIElementSetMessagingTimeout,
};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    string::{CFString, CFStringRef},
};

use super::errors::{check_ax_error, AXError};

/// Timeout in seconds for AX API calls to unresponsive apps.
const AX_MESSAGING_TIMEOUT_SECS: f32 = 1.0;

/// Owned wrapper around an `AXUIElementRef`.
///
/// Automatically retains on clone and releases on drop via Core Foundation's
/// reference counting (CFRetain / CFRelease are called by the `CFType` machinery).
pub struct AXElement {
    inner: CFType,
}

impl std::fmt::Debug for AXElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AXElement").finish_non_exhaustive()
    }
}

impl Clone for AXElement {
    fn clone(&self) -> Self {
        // wrap_under_get_rule increments the retain count.
        let inner = unsafe { CFType::wrap_under_get_rule(self.inner.as_CFTypeRef()) };
        Self { inner }
    }
}

// SAFETY: AXUIElementRef is thread-safe for read operations per Apple docs.
unsafe impl Send for AXElement {}
unsafe impl Sync for AXElement {}

impl AXElement {
    /// Wrap a raw `AXUIElementRef` (takes ownership; caller must not release separately).
    ///
    /// # Safety
    ///
    /// `raw` must be a valid, non-null `AXUIElementRef`.
    pub unsafe fn from_raw(raw: AXUIElementRef) -> Self {
        // SAFETY: caller guarantees `raw` is valid. `CFType::wrap_under_create_rule` takes
        // ownership of the +1 retain count that AX API functions return.
        let inner = unsafe { CFType::wrap_under_create_rule(raw as CFTypeRef) };
        Self { inner }
    }

    /// Create an application-level element from a PID.
    ///
    /// Sets a short messaging timeout so the tool does not hang on unresponsive apps.
    ///
    /// # Errors
    ///
    /// Does not fail directly, but AX calls on the returned element will fail with
    /// `AXError::Timeout` if the target app is unresponsive.
    pub fn application(pid: i32) -> Self {
        // SAFETY: `AXUIElementCreateApplication` returns a +1 retained ref. Always succeeds.
        let raw = unsafe { AXUIElementCreateApplication(pid) };
        // SAFETY: raw is always non-null.
        let el = unsafe { Self::from_raw(raw) };
        // Best-effort: set timeout. Ignore errors (element is valid regardless).
        // SAFETY: FFI call with a valid element ref.
        unsafe {
            AXUIElementSetMessagingTimeout(el.as_raw(), AX_MESSAGING_TIMEOUT_SECS);
        }
        el
    }

    /// Return the underlying raw pointer (not retained; valid only as long as `self` is alive).
    pub fn as_raw(&self) -> AXUIElementRef {
        self.inner.as_CFTypeRef() as AXUIElementRef
    }

    /// Get the PID of the process that owns this element.
    ///
    /// # Errors
    ///
    /// Returns `AXError` if the element is invalid.
    #[allow(dead_code)]
    pub fn pid(&self) -> Result<i32, AXError> {
        let mut pid: i32 = 0;
        // SAFETY: Safe FFI call. `pid` is a valid out-pointer.
        let code = unsafe { AXUIElementGetPid(self.as_raw(), &mut pid) };
        check_ax_error(code, "AXUIElementGetPid")?;
        Ok(pid)
    }

    /// Get the menu bar element for an application element.
    ///
    /// # Errors
    ///
    /// Returns `AXError::AttributeUnsupported` if the app has no standard menu bar
    /// (e.g., some Electron apps before they gain focus).
    pub fn menu_bar(&self) -> Result<AXElement, AXError> {
        self.copy_element_attribute(kAXMenuBarAttribute)
    }

    /// Get the extras (status bar / menu extras) menu bar for an application element.
    ///
    /// This is the right-side menu bar containing items like Wi-Fi, Bluetooth, etc.
    /// Each app owns its own extras; iterate all running apps to find all status items.
    ///
    /// # Errors
    ///
    /// Returns `AXError::AttributeUnsupported` if the app has no extras menu bar.
    pub fn extras_menu_bar(&self) -> Result<AXElement, AXError> {
        self.copy_element_attribute(kAXExtrasMenuBarAttribute)
    }

    /// Copy a single attribute value as an `AXElement`.
    fn copy_element_attribute(&self, attr: &'static str) -> Result<AXElement, AXError> {
        let attr_cf = CFString::from_static_string(attr);
        let mut value: CFTypeRef = std::ptr::null();
        let code = unsafe {
            AXUIElementCopyAttributeValue(self.as_raw(), attr_cf.as_concrete_TypeRef(), &mut value)
        };
        check_ax_error(code, attr)?;
        if value.is_null() {
            return Err(AXError::AttributeUnsupported(attr.to_owned()));
        }
        // SAFETY: value is a valid AXUIElementRef when the attribute is an element type.
        Ok(unsafe { AXElement::from_raw(value as AXUIElementRef) })
    }

    /// Get child elements (e.g., menu bar items or submenu items).
    ///
    /// # Errors
    ///
    /// Returns `AXError` if children cannot be fetched.
    pub fn children(&self) -> Result<Vec<AXElement>, AXError> {
        self.copy_array_attribute(kAXChildrenAttribute)
    }

    /// Get visible child elements (respects system hiding by Bartender/Ice).
    ///
    /// For extras menu bars, use this instead of `children()` to only get items
    /// the user can actually see (not hidden by menu bar managers).
    ///
    /// # Errors
    ///
    /// Returns `AXError` if visible children cannot be fetched.
    pub fn visible_children(&self) -> Result<Vec<AXElement>, AXError> {
        self.copy_array_attribute(kAXVisibleChildrenAttribute)
    }

    /// Copy an array attribute as a `Vec<AXElement>`.
    fn copy_array_attribute(&self, attr: &'static str) -> Result<Vec<AXElement>, AXError> {
        let attr_cf = CFString::from_static_string(attr);
        let mut value: CFTypeRef = std::ptr::null();
        let code = unsafe {
            AXUIElementCopyAttributeValue(self.as_raw(), attr_cf.as_concrete_TypeRef(), &mut value)
        };
        check_ax_error(code, attr)?;
        if value.is_null() {
            return Ok(Vec::new());
        }
        // SAFETY: AX children attribute always returns a CFArrayRef of AXUIElementRefs.
        let array = unsafe { CFArray::<CFType>::wrap_under_create_rule(value as CFArrayRef) };
        let mut result = Vec::with_capacity(array.len() as usize);
        for item in array.iter() {
            // Each item in the array is an AXUIElementRef (a CFTypeRef).
            let raw = item.as_CFTypeRef() as AXUIElementRef;
            // from_raw_retained calls wrap_under_get_rule, adding a retain so the element
            // stays alive beyond the array's lifetime.
            let el = unsafe { AXElement::from_raw_retained(raw) };
            result.push(el);
        }
        Ok(result)
    }

    /// Wrap a raw `AXUIElementRef`, adding a retain (for array elements we don't own outright).
    ///
    /// # Safety
    ///
    /// `raw` must be a valid, non-null `AXUIElementRef`.
    unsafe fn from_raw_retained(raw: AXUIElementRef) -> Self {
        // wrap_under_get_rule increments the reference count.
        let inner = unsafe { CFType::wrap_under_get_rule(raw as CFTypeRef) };
        Self { inner }
    }

    /// Perform an action on this element (e.g., `kAXPressAction`).
    ///
    /// # Errors
    ///
    /// Returns `AXError::ActionUnsupported` if the action is not available,
    /// or `AXError::InvalidElement` if the element is stale.
    pub fn perform_action(&self, action: &'static str) -> Result<(), AXError> {
        let action_cf = CFString::from_static_string(action);
        let code =
            unsafe { AXUIElementPerformAction(self.as_raw(), action_cf.as_concrete_TypeRef()) };
        check_ax_error(code, action)
    }

    /// Batch-fetch multiple attributes in a single IPC round-trip.
    ///
    /// Returns a parallel vec of `Option<AttributeValue>` — `None` if an attribute
    /// is not supported or has no value for this element.
    ///
    /// This is the primary performance optimization: instead of N sequential IPC calls,
    /// one call fetches all needed attributes.
    ///
    /// # Errors
    ///
    /// Returns `AXError` on API-level failure (not on per-attribute absence).
    pub fn batch_attributes(
        &self,
        attrs: &[&'static str],
    ) -> Result<Vec<Option<AttributeValue>>, AXError> {
        // Build a CFArray of CFString attribute names.
        let cf_attrs: Vec<CFString> = attrs
            .iter()
            .map(|&a| CFString::from_static_string(a))
            .collect();

        let cf_refs: Vec<*const core_foundation::string::__CFString> =
            cf_attrs.iter().map(|s| s.as_concrete_TypeRef()).collect();

        // SAFETY: CFArray::from_copyable creates a CFArray retaining each element.
        let attr_array = CFArray::from_copyable(&cf_refs);

        let mut out_array: CFArrayRef = std::ptr::null();
        let code = unsafe {
            AXUIElementCopyMultipleAttributeValues(
                self.as_raw(),
                attr_array.as_concrete_TypeRef(),
                0u32, // options: 0 = don't stop on error
                &mut out_array,
            )
        };

        // A non-success top-level code means the element itself is bad.
        if code != kAXErrorSuccess {
            check_ax_error(code, "AXUIElementCopyMultipleAttributeValues")?;
        }

        if out_array.is_null() {
            // Return all None
            return Ok(vec![None; attrs.len()]);
        }

        // SAFETY: out is a CFArrayRef of results, one per attribute.
        let result_array = unsafe { CFArray::<CFType>::wrap_under_create_rule(out_array) };

        let mut values = Vec::with_capacity(attrs.len());
        for item in result_array.iter() {
            let type_id = item.type_of();
            // AXValue errors come back as CFNumbers with the error code; we treat them as None.
            // Real values are CFString, CFBoolean, CFNumber, or AXUIElementRef (CFType).
            // We use type_of to distinguish.
            let parsed = parse_cf_type(&item, type_id);
            values.push(parsed);
        }

        Ok(values)
    }
}

/// A parsed attribute value from the AX API.
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// String attribute (e.g., title, role).
    String(String),
    /// Boolean attribute (e.g., enabled).
    Bool(bool),
    /// Number attribute (e.g., modifier mask).
    Number(i64),
    /// Child elements (from array attributes like `kAXChildrenAttribute`).
    #[allow(dead_code)]
    Elements(Vec<AXElement>),
}

/// Parse a `CFType` into an `AttributeValue`.
fn parse_cf_type(
    value: &CFType,
    type_id: core_foundation::base::CFTypeID,
) -> Option<AttributeValue> {
    use core_foundation::base::TCFType;

    // CFString type
    if type_id == CFString::type_id() {
        // SAFETY: We verified the type_id matches CFString.
        let s = unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as CFStringRef) };
        return Some(AttributeValue::String(s.to_string()));
    }

    // CFBoolean type
    if type_id == CFBoolean::type_id() {
        // SAFETY: Verified type_id.
        let b = unsafe { CFBoolean::wrap_under_get_rule(value.as_CFTypeRef() as *const _) };
        return Some(AttributeValue::Bool(b.into()));
    }

    // CFNumber type (modifier mask, etc.)
    if type_id == core_foundation::number::CFNumber::type_id() {
        use core_foundation::number::CFNumber;
        let n = unsafe { CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as *const _) };
        if let Some(v) = n.to_i64() {
            return Some(AttributeValue::Number(v));
        }
        return None;
    }

    // CFArray type (children)
    if type_id == CFArray::<CFType>::type_id() {
        let array =
            unsafe { CFArray::<CFType>::wrap_under_get_rule(value.as_CFTypeRef() as CFArrayRef) };
        let mut elements = Vec::with_capacity(array.len() as usize);
        for item in array.iter() {
            let raw = item.as_CFTypeRef() as AXUIElementRef;
            // SAFETY: Items in an AXChildren array are always AXUIElementRefs.
            let el = unsafe { AXElement::from_raw_retained(raw) };
            elements.push(el);
        }
        return Some(AttributeValue::Elements(elements));
    }

    // Unknown or error type (AX puts kAXError values as CFNumber — treated as None above).
    None
}

/// The standard set of attributes to fetch for each menu item in one batch call.
/// Order matters: results are indexed positionally.
pub const MENU_ITEM_ATTRS: &[&str] = &[
    kAXTitleAttribute,
    kAXEnabledAttribute,
    kAXMenuItemMarkCharAttribute,
    kAXMenuItemCmdCharAttribute,
    kAXMenuItemCmdModifiersAttribute,
    kAXRoleAttribute,
    kAXChildrenAttribute,
    kAXMenuItemPrimaryUIElementAttribute,
];

/// Indices into `MENU_ITEM_ATTRS`.
pub mod attr_idx {
    pub const TITLE: usize = 0;
    pub const ENABLED: usize = 1;
    pub const MARK_CHAR: usize = 2;
    pub const CMD_CHAR: usize = 3;
    pub const CMD_MODIFIERS: usize = 4;
    pub const ROLE: usize = 5;
    #[allow(dead_code)]
    pub const CHILDREN: usize = 6;
    /// Non-None when this item is an alternate of another item.
    pub const PRIMARY_UI_ELEMENT: usize = 7;
}
