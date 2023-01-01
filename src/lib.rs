mod helpers {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    pub fn to_wchar_vec(msg: &str) -> Vec<libc::wchar_t> {
        let wide_msg = widestring::U32CString::from_str(msg).expect("cannot convert to UTF-32/wchar_t");
        return wide_msg.into_vec_with_nul().iter().map(|&e| e as libc::wchar_t).collect();
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    pub fn toWcharVec(msg: &str) -> Vec<libc::wchar_t> {
        let wide_msg = widestring::U16CString::from_str(msg).expect("cannot convert to UTF-16/wchar_t");
        return wide_msg.into_vec_with_nul().iter().map(|&e| e as libc::wchar_t).collect();
    }
}

pub mod prt {
    use std::ffi::{c_char, c_void, CStr};

    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[repr(C)]
    enum Status {
        STATUS_OK,
        STATUS_UNSPECIFIED_ERROR,
        STATUS_OUT_OF_MEM,
        STATUS_NO_LICENSE,

        STATUS_NOT_ALL_IS_GENERATED,
        STATUS_INCOMPATIBLE_IS,

        STATUS_FILE_NOT_FOUND,
        STATUS_FILE_ALREADY_EXISTS,
        STATUS_COULD_NOT_OPEN_FILE,
        STATUS_COULD_NOT_CLOSE_FILE,
        STATUS_FILE_WRITE_FAILED,
        STATUS_FILE_READ_FAILED,
        STATUS_FILE_SEEK_FAILED,
        STATUS_FILE_TELL_FAILED,
        STATUS_NO_SEEK,
        STATUS_EMPTY_FILE,
        STATUS_INVALID_URI,

        STATUS_STREAM_ADAPTOR_NOT_FOUND,
        STATUS_RESOLVEMAP_PROVIDER_NOT_FOUND,
        STATUS_DECODER_NOT_FOUND,
        STATUS_ENCODER_NOT_FOUND,
        STATUS_UNABLE_TO_RESOLVE,
        STATUS_CHECK_ERROR_PARAM,
        STATUS_KEY_NOT_FOUND,
        STATUS_KEY_ALREADY_TAKEN,
        STATUS_KEY_NOT_SUPPORTED,
        STATUS_STRING_TRUNCATED,
        STATUS_ILLEGAL_CALLBACK_OBJECT,
        STATUS_ILLEGAL_LOG_HANDLER,
        STATUS_ILLEGAL_LOG_LEVEL,
        STATUS_ILLEGAL_VALUE,
        STATUS_NO_RULEFILE,
        STATUS_NO_INITIAL_SHAPE,
        STATUS_CGB_ERROR,
        STATUS_NOT_INITIALIZED,
        STATUS_ALREADY_INITIALIZED,
        STATUS_INCONSISTENT_TEXTURE_PARAMS,
        STATUS_CANCELED,
        STATUS_UNKNOWN_ATTRIBUTE,
        STATUS_UNKNOWN_RULE,
        STATUS_ARGUMENTS_MISMATCH,
        STATUS_BUFFER_TO_SMALL,
        STATUS_UNKNOWN_FORMAT,
        STATUS_ENCODE_FAILED,
        STATUS_ATTRIBUTES_ALREADY_SET,
        STATUS_ATTRIBUTES_NOT_SET,
        STATUS_GEOMETRY_ALREADY_SET,
        STATUS_GEOMETRY_NOT_SET,
        STATUS_ILLEGAL_GEOMETRY,
        STATUS_NO_GEOMETRY,
    }

    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[repr(C)]
    pub enum LogLevel {
        LOG_TRACE = 0,
        LOG_DEBUG = 1,
        LOG_INFO = 2,
        LOG_WARNING = 3,
        LOG_ERROR = 4,
        LOG_FATAL = 5,
        LOG_NO = 1000,
    }

    #[repr(C)]
    struct Version {
        version_major: i32,
        version_minor: i32,
        version_build: i32,

        name: *const c_char,
        full_name: *const c_char,
        version: *const c_char,
        build_config: *const c_char,
        build_os: *const c_char,
        build_arch: *const c_char,
        build_tc: *const c_char,
        build_date: *const c_char,

        name_w: *const libc::wchar_t,
        full_name_w: *const libc::wchar_t,
        version_w: *const libc::wchar_t,
        build_config_w: *const libc::wchar_t,
        build_os_w: *const libc::wchar_t,
        build_arch_w: *const libc::wchar_t,
        build_tc_w: *const libc::wchar_t,
        build_date_w: *const libc::wchar_t,

        cga_version_major: i32,
        cga_version_minor: i32,
        cga_version: *const c_char,
        cga_version_w: *const i32,

        cgac_version_major: i32,
        cgac_version_minor: i32,
        cgac_version: *const c_char,
        cgac_version_w: *const i32,
    }

    #[repr(C)]
    struct Object {
        dummy: i8, // to avoid the "unsafe FFI object" warning
    }

    impl Object {
        fn destroy(&self) {}
    }

    extern "C" {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt4initEPKPKwmNS_8LogLevelEPNS_6StatusE"]
        fn init(prt_plugins: *const *const libc::wchar_t, prt_plugins_count: libc::size_t,
                log_level: LogLevel) -> *const Object;

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt10getVersionEv"]
        fn get_version() -> *const Version;

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt20getStatusDescriptionENS_6StatusE"]
        fn get_status_description(input: i32) -> *const c_char;
    }

    pub trait LogHandler {
        fn handle_log_event(&mut self, msg: &str);
    }

    #[repr(C)]
    struct AbstractLogHandlerBinding<T> where T: LogHandler {
        handle_log_event: unsafe extern fn(*mut T, msg: *const c_char),
        context: *mut T,
    }

    #[derive(Default)]
    pub struct DefaultLogHandler {}

    impl LogHandler for DefaultLogHandler {
        fn handle_log_event(&mut self, msg: &str) {
            println!("{}", msg);
        }
    }

    #[link(name = "bindings", kind = "static")]
    extern "C" {
        fn ffi_add_log_handler(log_handler: *mut c_void);
        fn ffi_remove_log_handler(log_handler: *mut c_void);
    }

    pub fn add_log_handler<T>(log_handler: &mut Box<T>) where T: LogHandler {
        unsafe extern "C" fn handle_log_event<T>(context: *mut T, cmsg: *const c_char)
            where T: LogHandler
        {
            let handler_ref: &mut T = &mut *context;

            let msg = CStr::from_ptr(cmsg).to_str().unwrap();
            handler_ref.handle_log_event(msg);
        }

        let context: *mut T = log_handler.as_mut();
        let binding: Box<AbstractLogHandlerBinding<T>> = Box::new(AbstractLogHandlerBinding {
            handle_log_event,
            context,
        });

        let binding_ptr: *mut c_void = Box::into_raw(binding) as *mut c_void;
        unsafe {
            ffi_add_log_handler(binding_ptr);
        }
    }

    extern "C" {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt3logEPKwNS_8LogLevelE"]
        fn prt_log(msg: *const libc::wchar_t, level: LogLevel);
    }

    pub fn log(msg: &str, level: LogLevel) {
        let cs_vec = crate::helpers::to_wchar_vec(msg);
        unsafe {
            prt_log(cs_vec.as_ptr(), level);
        }
    }

    pub fn remove_log_handler<T>(log_handler: &mut Box<T>) where T: LogHandler {
        unsafe extern "C" fn handle_log_event<T>(_context: *mut T, _cmsg: *const c_char)
            where T: LogHandler
        {}

        let context = log_handler.as_mut() as *mut T;
        let binding: Box<AbstractLogHandlerBinding<T>> = Box::new(AbstractLogHandlerBinding {
            handle_log_event,
            context,
        });

        let binding_ptr: *mut c_void = Box::into_raw(binding) as *mut c_void;
        unsafe {
            ffi_remove_log_handler(binding_ptr);
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{fs, slice};
        use std::ffi::{CStr, CString};
        use std::path::PathBuf;
        use super::*;

        fn as_vec_u16(v: &[i32]) -> Vec<u16> {
            let mut out: Vec<u16> = Vec::with_capacity(v.len());
            for i in v {
                out.push(*i as u16);
            }
            return out;
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))] // assuming 4 bytes for wchar_t
        unsafe fn as_string_from_wchar_array(ptr: *const libc::wchar_t) -> String {
            assert!(!ptr.is_null());
            let ptr_len = libc::wcslen(ptr);
            assert!(ptr_len > 0);
            let ptr_slice: &[i32] = slice::from_raw_parts(ptr, ptr_len as usize);
            let raw_utf16: Vec<u16> = as_vec_u16(ptr_slice);
            return String::from_utf16(raw_utf16.as_slice()).unwrap_or_default();
        }

        fn from_cchar_ptr_to_str(cchar_ptr: *const c_char) -> String {
            let val_cstr = unsafe { CStr::from_ptr(cchar_ptr) };
            return val_cstr.to_str().unwrap_or_default().to_string(); // this is probably far from ideal ;-)
        }

        fn print_and_assert_cstring(prefix: &str, raw_val: *const c_char, expected_val: &str) {
            let string_val = from_cchar_ptr_to_str(raw_val);
            println!("{} = {}", prefix, string_val);
            assert_eq!(string_val, expected_val);
        }

        fn print_and_assert_string(prefix: &str, raw_val: String, expected_val: &str) {
            println!("{} = {}", prefix, raw_val);
            assert_eq!(raw_val, expected_val);
        }

        fn print_and_assert_int(prefix: &str, raw_val: i32, expected_val: i32) {
            println!("{} = {}", prefix, raw_val);
            assert_eq!(raw_val, expected_val);
        }

        #[test]
        fn prt_get_version() {
            unsafe {
                let ver = &*get_version();
                print_and_assert_cstring("prt::Version::mName", ver.name, "ArcGIS Procedural Runtime");
                print_and_assert_cstring("prt::Version::mVersion", ver.version, "2.7.8538");
                print_and_assert_cstring("prt::Version::mBuildConfig", ver.build_config, "PRT_BC_REL");
                print_and_assert_cstring("prt::Version::mBuildOS", ver.build_os, "linux");
                print_and_assert_cstring("prt::Version::mBuildArch", ver.build_arch, "x86_64");
                print_and_assert_cstring("prt::Version::mBuildTC", ver.build_tc, "PRT_TC_GCC93");
                print_and_assert_cstring("prt::Version::mBuildDate", ver.build_date, "2022-10-04 15:48");

                let name = as_string_from_wchar_array(ver.name_w);
                print_and_assert_string("prt::Version::mwName", name, "ArcGIS Procedural Runtime");

                let build_date = as_string_from_wchar_array(ver.build_date_w);
                print_and_assert_string("prt::Version::mwBuildDate", build_date, "2022-10-04 15:48");

                print_and_assert_int("prt::Version::mCGAVersionMajor", ver.cga_version_major, 2022);
                print_and_assert_int("prt::Version::mCGAVersionMinor", ver.cga_version_minor, 1);

                print_and_assert_int("prt::Version::mCGACVersionMajor", ver.cgac_version_major, 1);
                print_and_assert_int("prt::Version::mCGACVersionMinor", ver.cgac_version_minor, 19);
            }
        }

        #[test]
        fn prt_get_status_description() {
            unsafe {
                let status_description_cchar_ptr = get_status_description(2);

                let status_description_cstr = CStr::from_ptr(status_description_cchar_ptr);
                let status_description = status_description_cstr.to_str().unwrap_or_default();

                println!("prt::getStatusDescription(2) = {}", status_description);
                assert_eq!(status_description, "Out of memory.");
            }
        }

        #[test]
        fn prt_init() {
            let crate_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
            let deps_path = crate_root.join("deps");
            let cesdk_path_entry = fs::read_dir(deps_path).expect("cannot read deps dir")
                .filter(|p| p.is_ok() && p.as_ref().unwrap().file_type().unwrap().is_dir())
                .find(|p| p.as_ref().unwrap().path().file_name().unwrap().to_str().unwrap().starts_with("esri_ce_sdk-"));
            let cesdk_lib_path = cesdk_path_entry.expect("could not find cesdk lib dir").unwrap().path().join("lib");

            let cesdk_lib_dir_c = CString::new(cesdk_lib_path.to_str().expect("foo")).expect("CString::new failed");
            let cesdk_lib_dir_wchar: Vec<libc::wchar_t> = cesdk_lib_dir_c.as_bytes_with_nul().iter().map(|&e| e as libc::wchar_t).collect();
            let plugins_dirs: [*const libc::wchar_t; 1] = [cesdk_lib_dir_wchar.as_ptr()];

            unsafe {
                let prt_handle = init(plugins_dirs.as_ptr(), plugins_dirs.len(), LogLevel::LOG_DEBUG);
                Object::destroy(&*prt_handle);
            }
        }
    }
}
