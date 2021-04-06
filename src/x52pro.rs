use libloading::{Library, Symbol};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::ctypes::wchar_t;
use winapi::um::winnt::HRESULT;

type DirectOutputInitializeFn = unsafe extern "C" fn(wszPluginName: *const wchar_t) -> HRESULT;

const PLUGIN_NAME: &str = "EDXLC";

pub struct DirectOutput {
    library: Library,
}

impl DirectOutput {
    pub fn load() -> DirectOutput {
        DirectOutput {
            library: DirectOutput::load_library(),
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

    unsafe fn load_library_function<T>(&self, function_name: &[u8]) -> Symbol<T> {
        self.library.get(function_name).unwrap()
    }

    fn win32_string(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }
}
