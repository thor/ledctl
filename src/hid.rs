use std::fmt;
use std::os::raw::c_int;

pub(crate) type IOReturn = c_int;

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
