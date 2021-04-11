use libc::c_void;
use libloading::{Library, Symbol};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::DWORD;
use winapi::um::winnt::HRESULT;

type DirectOutputDeviceHandle = *const c_void;

type DirectOutputInitializeFn = unsafe extern "C" fn(wszPluginName: *const wchar_t) -> HRESULT;
type DirectOutputEnumerateFn = unsafe extern "C" fn(
    pfnCb: DirectOutputEnumerateCallbackFn,
    pCtxt: &mut DirectOutput,
) -> HRESULT;
type DirectOutputEnumerateCallbackFn =
    extern "C" fn(hDevice: DirectOutputDeviceHandle, pCtxt: &mut DirectOutput);
type DirectOutputAddPageFn = unsafe extern "C" fn(
    hDevice: DirectOutputDeviceHandle,
    dwPage: DWORD,
    wszDebugName: *const wchar_t,
    dwFlags: DWORD,
) -> HRESULT;
type DirectOutputSetLedFn = unsafe extern "C" fn(
    hDevice: DirectOutputDeviceHandle,
    dwPage: DWORD,
    dwIndex: DWORD,
    dwValue: DWORD,
) -> HRESULT;

const FLAG_SET_AS_ACTIVE: DWORD = 1;

const PLUGIN_NAME: &str = "EDXLC";
const PAGE_ID: DWORD = 1;

pub struct DirectOutput {
    library: Library,
    device: DirectOutputDeviceHandle,
}

impl DirectOutput {
    pub fn load() -> DirectOutput {
        DirectOutput {
            library: DirectOutput::load_library(),
            device: std::ptr::null(),
        }
    }

    fn load_library() -> Library {
        unsafe {
            Library::new(r"C:\Program Files\Logitech\DirectOutput\DirectOutput.dll")
                .expect("Could not load DirectOutput.dll")
        }
    }

    pub fn initialize(&self) {
        unsafe {
            let initialize_fn =
                self.load_library_function::<DirectOutputInitializeFn>(b"DirectOutput_Initialize");
            let result = initialize_fn(DirectOutput::win32_string(PLUGIN_NAME).as_ptr());
            println!("DirectOutput_Initialize result = {:?}", result);

            if result != 0 {
                panic!("Could not initialize the DirectOutput library");
            }
        }
    }

    pub fn enumerate(&mut self) {
        extern "C" fn callback(device: DirectOutputDeviceHandle, target: &mut DirectOutput) {
            println!("DirectOutput_Enumerate device = {:?}", device);
            target.device = device;
        }

        unsafe {
            let enumerate_fn =
                self.load_library_function::<DirectOutputEnumerateFn>(b"DirectOutput_Enumerate");
            let result = enumerate_fn(callback, self);
            println!("DirectOutput_Enumerate result = {:?}", result);

            if result != 0 {
                panic!("Could not enumerate dervices with DirectOutput");
            }
        }
    }

    pub fn add_page(&self) {
        // Despite what the SDK documentation says, we have to pass in a non-null debug
        // name or later calls fail with an error indicating the page is not active.
        let debug_name = DirectOutput::win32_string(PLUGIN_NAME).as_ptr();

        unsafe {
            let add_page_fn =
                self.load_library_function::<DirectOutputAddPageFn>(b"DirectOutput_AddPage");
            let result = add_page_fn(self.device, PAGE_ID, debug_name, FLAG_SET_AS_ACTIVE);
            println!("DirectOutput_AddPage result = {:?}", result);

            if result != 0 {
                panic!("Could not add page with DirectOutput");
            }
        }
    }

    pub fn set_led(&self, id: u32, active: bool) {
        let value = if active { 1 } else { 0 };

        unsafe {
            // We should not be re-loading this function symbol on every call.
            let set_led_fn =
                self.load_library_function::<DirectOutputSetLedFn>(b"DirectOutput_SetLed");
            let result = set_led_fn(self.device, PAGE_ID, id, value);
            println!("DirectOutput_SetLed result = {:?}", result);

            if result != 0 {
                panic!("Could not set LED with DirectOutput");
            }
        }
    }

    unsafe fn load_library_function<T>(&self, function_name: &[u8]) -> Symbol<T> {
        self.library.get(function_name).unwrap()
    }

    fn win32_string(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }
}
