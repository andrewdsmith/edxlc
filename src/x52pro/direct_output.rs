use libc::c_void;
use libloading::{os::windows::Symbol, Library};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::DWORD;
use winapi::um::winnt::HRESULT;

type DeviceHandle = *const c_void;

type InitializeFn = unsafe extern "C" fn(wszPluginName: *const wchar_t) -> HRESULT;
type EnumerateFn =
    unsafe extern "C" fn(pfnCb: EnumerateCallbackFn, pCtxt: &mut DirectOutput) -> HRESULT;
type EnumerateCallbackFn = extern "C" fn(hDevice: DeviceHandle, pCtxt: &mut DirectOutput);
type AddPageFn = unsafe extern "C" fn(
    hDevice: DeviceHandle,
    dwPage: DWORD,
    wszDebugName: *const wchar_t,
    dwFlags: DWORD,
) -> HRESULT;
type SetLedFn = unsafe extern "C" fn(
    hDevice: DeviceHandle,
    dwPage: DWORD,
    dwIndex: DWORD,
    dwValue: DWORD,
) -> HRESULT;

const FLAG_SET_AS_ACTIVE: DWORD = 1;

const PLUGIN_NAME: &str = "EDXLC";
const PAGE_ID: DWORD = 1;

/// An instance of a safe wrapper around the Saitek DirectOutput library.
pub struct DirectOutput {
    // We have to continue to own the Library instance even though we never use
    // it again so that it is not dropped and hence closed, which would
    // invalidate the symbols loaded from it we want to use to call functions.
    #[allow(dead_code)]
    library: Library,
    initialize_fn: Symbol<InitializeFn>,
    enumerate_fn: Symbol<EnumerateFn>,
    add_page_fn: Symbol<AddPageFn>,
    set_led_fn: Symbol<SetLedFn>,
    device: DeviceHandle,
}

impl DirectOutput {
    /// Returns a new instance of the library loaded from its default
    /// installation location. Panics is the library cannot be loaded, e.g. not
    /// installed at the given location.
    pub fn load() -> DirectOutput {
        let library = DirectOutput::load_library();
        let initialize_fn = DirectOutput::get_library_symbol(&library, b"DirectOutput_Initialize");
        let enumerate_fn = DirectOutput::get_library_symbol(&library, b"DirectOutput_Enumerate");
        let add_page_fn = DirectOutput::get_library_symbol(&library, b"DirectOutput_AddPage");
        let set_led_fn = DirectOutput::get_library_symbol(&library, b"DirectOutput_SetLed");

        DirectOutput {
            library,
            initialize_fn,
            enumerate_fn,
            add_page_fn,
            set_led_fn,
            device: std::ptr::null(),
        }
    }

    fn load_library() -> Library {
        unsafe {
            Library::new(r"C:\Program Files\Logitech\DirectOutput\DirectOutput.dll")
                .expect("Could not load DirectOutput.dll")
        }
    }

    /// Given a function name returns a symbol for that function in the
    /// DirectOutput library. Panics if the symbol cannot be found.
    fn get_library_symbol<T>(library: &Library, symbol: &[u8]) -> Symbol<T> {
        unsafe { library.get::<T>(symbol).unwrap().into_raw() }
    }

    /// Initializes the underlying library. This must be called before any
    /// other methods can be called. Panics if the initialization fails.
    pub fn initialize(&self) {
        unsafe {
            let result = (self.initialize_fn)(DirectOutput::win32_string(PLUGIN_NAME).as_ptr());
            println!("DirectOutput_Initialize result = {:?}", result);

            if result != 0 {
                panic!("Could not initialize the DirectOutput library");
            }
        }
    }

    /// Enumerates the connected Saitek devices and selects the last given
    /// device. This wrapper does not give the ability to select a device by
    /// type or id but could be extended to do so. For the purposes of this
    /// project it is currently assuming that only X52Pro devices are attached,
    /// which may not be true in general. Panics if the enumeration fails.
    pub fn enumerate(&mut self) {
        extern "C" fn callback(device: DeviceHandle, target: &mut DirectOutput) {
            println!("DirectOutput_Enumerate device = {:?}", device);
            target.device = device;
        }

        unsafe {
            let result = (self.enumerate_fn)(callback, self);
            println!("DirectOutput_Enumerate result = {:?}", result);

            if result != 0 {
                panic!("Could not enumerate dervices with DirectOutput");
            }
        }
    }

    /// Adds a display page to the device. This method must be called after
    /// `initialize` and before `set_led`. The underlying library supports
    /// multiple display pages that can be switched between but this wrapper
    /// creates a single page only. Panics if the addition fails.
    pub fn add_page(&self) {
        // Despite what the SDK documentation says, we have to pass in a non-null debug
        // name or later calls fail with an error indicating the page is not active.
        let debug_name = DirectOutput::win32_string(PLUGIN_NAME).as_ptr();

        unsafe {
            let result = (self.add_page_fn)(self.device, PAGE_ID, debug_name, FLAG_SET_AS_ACTIVE);
            println!("DirectOutput_AddPage result = {:?}", result);

            if result != 0 {
                panic!("Could not add page with DirectOutput");
            }
        }
    }

    /// Activates or deactives the LED with the given `id` on the joystick. The
    /// `id` must be between 0 and 19 inclusive for the X52Pro. Panics if
    /// setting the LED state fails, e.g. if given an invalid `id`.
    pub fn set_led(&self, id: u32, active: bool) {
        let value = if active { 1 } else { 0 };

        unsafe {
            let result = (self.set_led_fn)(self.device, PAGE_ID, id, value);
            println!("DirectOutput_SetLed result = {:?}", result);

            if result != 0 {
                panic!("Could not set LED with DirectOutput");
            }
        }
    }

    /// Given a native string `value` returns a Windows native "wide" string
    /// suitable for passing to Windows-native code.
    fn win32_string(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }
}
