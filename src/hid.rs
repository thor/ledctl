use std::fmt;
use std::os::raw::c_int;

use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{CFIndex, CFRelease, CFTypeRef};
use core_foundation_sys::dictionary::{
    CFDictionaryAddValue, CFDictionaryCreateMutable, CFMutableDictionaryRef,
    kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
};
use core_foundation_sys::number::{
    CFNumberCreate, CFNumberGetValue, CFNumberRef, kCFNumberSInt32Type,
};
use core_foundation_sys::set::{CFSetGetCount, CFSetGetValues, CFSetRef};
use core_foundation_sys::string::{
    CFStringCreateWithCString, CFStringGetCString, CFStringGetLength, CFStringRef,
    kCFStringEncodingUTF8,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

pub(crate) type IOReturn = c_int;

pub(crate) type IOOptionBits = u32;
pub(crate) type IOHIDManagerRef = *mut c_void;
pub(crate) type IOHIDDeviceRef = *mut c_void;
pub(crate) type IOHIDElementRef = *mut c_void;
pub(crate) type IOHIDValueRef = *mut c_void;

const K_IO_RETURN_SUCCESS: IOReturn = 0;
const K_IO_HID_OPTIONS_TYPE_NONE: IOOptionBits = 0;

const K_HID_PAGE_GENERIC_DESKTOP: u32 = 0x01;
const K_HID_USAGE_GD_KEYBOARD: u32 = 0x06;
const K_HID_PAGE_LEDS: u32 = 0x08;

const K_IO_HID_PRODUCT_KEY: &str = "Product";
const K_IO_HID_LOCATION_ID_KEY: &str = "LocationID";
const K_IO_HID_DEVICE_USAGE_PAGE_KEY: &str = "DeviceUsagePage";
const K_IO_HID_DEVICE_USAGE_KEY: &str = "DeviceUsage";
const K_IO_HID_ELEMENT_USAGE_PAGE_KEY: &str = "UsagePage";

extern "C" {
    fn IOHIDManagerCreate(allocator: *const c_void, options: IOOptionBits) -> IOHIDManagerRef;
    fn IOHIDManagerOpen(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    fn IOHIDManagerClose(manager: IOHIDManagerRef, options: IOOptionBits) -> IOReturn;
    fn IOHIDManagerSetDeviceMatching(manager: IOHIDManagerRef, matching: *const c_void);
    fn IOHIDManagerCopyDevices(manager: IOHIDManagerRef) -> CFSetRef;

    fn IOHIDDeviceGetProperty(device: IOHIDDeviceRef, key: CFStringRef) -> *const c_void;
    fn IOHIDDeviceConformsTo(device: IOHIDDeviceRef, usage_page: u32, usage: u32) -> u8;
    fn IOHIDDeviceCopyMatchingElements(
        device: IOHIDDeviceRef,
        matching: *const c_void,
        options: IOOptionBits,
    ) -> CFArrayRef;
    fn IOHIDDeviceGetValue(
        device: IOHIDDeviceRef,
        element: IOHIDElementRef,
        value: *mut IOHIDValueRef,
    ) -> IOReturn;
    fn IOHIDDeviceSetValue(
        device: IOHIDDeviceRef,
        element: IOHIDElementRef,
        value: IOHIDValueRef,
    ) -> IOReturn;

    fn IOHIDValueCreateWithIntegerValue(
        allocator: *const c_void,
        element: IOHIDElementRef,
        timestamp: u64,
        value: CFIndex,
    ) -> IOHIDValueRef;
    fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> CFIndex;

    fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
}

unsafe fn cf_string(s: &str) -> CFStringRef {
    let c = CString::new(s).expect("interior nul in CF string key");
    CFStringCreateWithCString(ptr::null(), c.as_ptr(), kCFStringEncodingUTF8)
}

unsafe fn cf_string_to_rust(s: CFStringRef) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let len = CFStringGetLength(s);
    let max_buf = len * 4 + 1;
    let mut buf: Vec<c_char> = vec![0; max_buf as usize];
    if CFStringGetCString(s, buf.as_mut_ptr(), max_buf, kCFStringEncodingUTF8) == 0 {
        return None;
    }
    Some(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
}

unsafe fn cf_number_i32(n: i32) -> CFNumberRef {
    CFNumberCreate(ptr::null(), kCFNumberSInt32Type, &n as *const i32 as *const c_void)
}

unsafe fn make_device_matching_dict(usage_page: i32, usage: i32) -> CFMutableDictionaryRef {
    let dict = CFDictionaryCreateMutable(
        ptr::null(),
        0,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks,
    );
    let key_page = cf_string(K_IO_HID_DEVICE_USAGE_PAGE_KEY);
    let key_usage = cf_string(K_IO_HID_DEVICE_USAGE_KEY);
    let val_page = cf_number_i32(usage_page);
    let val_usage = cf_number_i32(usage);
    CFDictionaryAddValue(dict, key_page as *const c_void, val_page as *const c_void);
    CFDictionaryAddValue(dict, key_usage as *const c_void, val_usage as *const c_void);
    CFRelease(key_page as CFTypeRef);
    CFRelease(key_usage as CFTypeRef);
    CFRelease(val_page as CFTypeRef);
    CFRelease(val_usage as CFTypeRef);
    dict
}

unsafe fn make_element_matching_dict(usage_page: i32) -> CFMutableDictionaryRef {
    let dict = CFDictionaryCreateMutable(
        ptr::null(),
        0,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks,
    );
    let key_page = cf_string(K_IO_HID_ELEMENT_USAGE_PAGE_KEY);
    let val_page = cf_number_i32(usage_page);
    CFDictionaryAddValue(dict, key_page as *const c_void, val_page as *const c_void);
    CFRelease(key_page as CFTypeRef);
    CFRelease(val_page as CFTypeRef);
    dict
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Led {
    Caps,
    Num,
    Scroll,
}

impl Led {
    pub fn usage(self) -> u32 {
        match self {
            Led::Caps => 0x02,
            Led::Num => 0x01,
            Led::Scroll => 0x03,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedState {
    pub caps: bool,
    pub num: bool,
    pub scroll: bool,
}

#[derive(Debug)]
pub enum LedError {
    ManagerOpenFailed(IOReturn),
    DeviceNotFound,
    ElementNotFound(Led),
    IoKitError(IOReturn),
}

impl fmt::Display for LedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedError::ManagerOpenFailed(r) => write!(f, "Failed to open HID manager (IOReturn {:#x})", r),
            LedError::DeviceNotFound => write!(f, "No matching keyboard device found"),
            LedError::ElementNotFound(led) => write!(f, "LED element not found on device: {:?}", led),
            LedError::IoKitError(r) => write!(f, "IOKit error (IOReturn {:#x})", r),
        }
    }
}

impl std::error::Error for LedError {}

pub struct HidSession;
