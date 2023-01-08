pub mod prt {
    use std::{collections, path};
    use std::ffi;
    use std::fmt::{Display, Formatter};
    use std::ptr;
    use std::ptr::{null, null_mut};

    #[derive(Debug)]
    pub struct PrtError {
        pub message: String,
        pub status: Option<Status>,
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
        fn ffi_init(prt_plugins: *const *const libc::wchar_t,
                    prt_plugins_count: libc::size_t,
                    log_level: LogLevel,
                    status: *mut Status) -> *const Object;
    }

    pub struct PrtContext {
        handle: *const Object,
    }

    unsafe impl Sync for PrtContext {} // handle is thread-safe

    impl Drop for PrtContext {
        fn drop(&mut self) {
            unsafe {
                Object::destroy(&*self.handle);
            }
        }
    }

    impl Display for PrtContext {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "PrtContext native handle at {:p}", self.handle)
        }
    }

    pub fn init(_extra_plugin_paths: Option<Vec<path::PathBuf>>,
                initial_minimal_log_level: Option<LogLevel>) -> Result<Box<PrtContext>, PrtError>
    {
        // we include the built-in extension path by default
        let cesdk_lib_path = crate::helpers::get_cesdk_path().join("lib");
        if !cesdk_lib_path.exists() {
            return Err(PrtError {
                message: format!("Error while loading built-in extensions: {}", get_status_description(Status::STATUS_FILE_NOT_FOUND)),
                status: Some(Status::STATUS_FILE_NOT_FOUND),
            });
        }
        let cesdk_lib_dir_wchar_vec = crate::helpers::from_string_to_wchar_vec(cesdk_lib_path.to_str().unwrap());

        // append additional extension dirs
        // TODO: handle extra_plugin_paths

        let plugins_dirs: [*const libc::wchar_t; 1] = [cesdk_lib_dir_wchar_vec.as_ptr()];
        let log_level = initial_minimal_log_level.or(Some(LogLevel::LOG_WARNING));
        unsafe {
            let mut status = Status::STATUS_UNSPECIFIED_ERROR;
            let prt_handle = ffi_init(plugins_dirs.as_ptr(),
                                      plugins_dirs.len(),
                                      log_level.unwrap(),
                                      ptr::addr_of_mut!(status));
            return if (prt_handle != ptr::null()) && (status == Status::STATUS_OK) {
                Ok(Box::new(PrtContext { handle: prt_handle }))
            } else {
                // TODO: add details and prt::Status handling
                Err(PrtError {
                    message: get_status_description(Status::STATUS_UNSPECIFIED_ERROR),
                    status: Some(Status::STATUS_UNSPECIFIED_ERROR),
                })
            };
        }
    }

    #[derive(PartialEq)]
    #[derive(Debug)]
    pub enum PrimitiveType {
        Undefined(),
        String(String),
        Float(f64),
        Bool(bool),
        Int(i32),
        StringArray(Vec<String>),
        FloatArray(Vec<f64>),
        BoolArray(Vec<bool>),
        IntArray(Vec<i32>),
    }

    pub type EncoderOptions = collections::HashMap<String, PrimitiveType>;

    #[repr(C)]
    struct AttributeMap {
        dummy: i32,
    }

    #[repr(C)]
    struct ResolveMap {
        dummy: i32,
    }

    #[repr(C)]
    struct InitialShapeWrapper {
        vertex_coords: *const f64,
        vertex_coords_count: libc::size_t,
        indices: *const u32,
        indices_count: libc::size_t,
        face_counts: *const u32,
        face_counts_count: libc::size_t,

        rule_file: *const ffi::c_char,
        start_rule: *const ffi::c_char,
        random_seed: i32,
        name: *const ffi::c_char,
        attributes: *const AttributeMap,
        resolve_map: *const ResolveMap,
    }

    // TODO: add builder methods
    pub struct InitialShape {
        pub vertex_coords: Vec<f64>,
        pub indices: Vec<u32>,
        pub face_counts: Vec<u32>,

        pub rule_file: ffi::CString,
        pub start_rule: ffi::CString,
        pub random_seed: i32,
        pub name: ffi::CString,
    }

    impl InitialShape {
        //     fn vertex_coords(&mut self, coords: Vec<f64>) {
        //         self.vertex_coords = coords;
        //     }
        fn get_wrapper(&self) -> InitialShapeWrapper {
            return InitialShapeWrapper {
                vertex_coords: self.vertex_coords.as_ptr(),
                vertex_coords_count: self.vertex_coords.len(),
                indices: self.indices.as_ptr(),
                indices_count: self.indices.len(),
                face_counts: self.face_counts.as_ptr(),
                face_counts_count: self.face_counts.len(),
                rule_file: self.rule_file.as_ptr(),
                start_rule: self.start_rule.as_ptr(),
                random_seed: self.random_seed,
                name: self.name.as_ptr(),
                attributes: null(), // TODO
                resolve_map: null(), // TODO
            };
        }
    }

    pub trait Callbacks {}

    #[repr(C)]
    struct AbstractCallbacksBinding<T> where T: Callbacks {
        context: *mut T,
    }

    #[derive(Default)]
    pub struct FileCallbacks {}

    impl Callbacks for FileCallbacks {}

    #[repr(C)]
    struct Cache {
        dummy: i32,
    }

    #[repr(C)]
    struct OcclusionSet {
        dummy: i32,
    }

    #[link(name = "bindings", kind = "static")]
    extern "C" {
        fn ffi_generate(initial_shapes: *const *const InitialShapeWrapper,
                        initial_shapes_count: libc::size_t,
                        occlusion_handles: *const u64, // see prt::OcclusionSet::Handle
                        encoders: *const *const libc::wchar_t,
                        encoders_count: libc::size_t,
                        encoder_options: *const AttributeMap,
                        callbacks: *mut ffi::c_void,
                        cache: *mut Cache,
                        occl_set: *const OcclusionSet,
                        generate_options: *const AttributeMap) -> Status;
    }

    pub fn generate<C>(initial_shapes: &Vec<Box<InitialShape>>,
                       encoders: &Vec<String>,
                       encoder_options: &Vec<EncoderOptions>,
                       callbacks: &mut Box<C>) -> Status // todo: consistent error handling
        where C: Callbacks
    {
        if encoders.len() != encoder_options.len() {
            return Status::STATUS_ARGUMENTS_MISMATCH;
        }

        unsafe {
            let initial_shape_wrappers: Vec<InitialShapeWrapper> = initial_shapes.iter()
                .map(|x| x.get_wrapper())
                .collect();
            let initial_shape_wrapper_ptr_vec: Vec<*const InitialShapeWrapper> = initial_shape_wrappers.iter()
                .map(|x| &*x as *const InitialShapeWrapper).collect();

            let occlusion_handles: *const u64 = null();

            // TODO: probably better to stay UTF-8 on the rust side and convert in the native wrapper
            let encoders_wchar_vec: Vec<Vec<libc::wchar_t>> = encoders.iter()
                .map(|x| crate::helpers::from_string_to_wchar_vec(x.as_str()))
                .collect();
            let encoders_ptr_vec: Vec<*const libc::wchar_t> = encoders_wchar_vec.iter().map(|x| x.as_ptr()).collect();

            let encoder_options_ptr_vec: *const AttributeMap = null();

            let callbacks_context: *mut C = callbacks.as_mut();
            let callbacks_binding: Box<AbstractCallbacksBinding<C>>
                = Box::new(AbstractCallbacksBinding { context: callbacks_context });
            let callbacks_binding_ptr
                = Box::into_raw(callbacks_binding) as *mut ffi::c_void;

            let cache: *mut Cache = null_mut();
            let occl_set: *const OcclusionSet = null();
            let generate_options: *const AttributeMap = null();

            let status = ffi_generate(initial_shape_wrapper_ptr_vec.as_ptr(),
                                      initial_shape_wrapper_ptr_vec.len(),
                                      occlusion_handles,
                                      encoders_ptr_vec.as_ptr(), encoders_ptr_vec.len(),
                                      encoder_options_ptr_vec,
                                      callbacks_binding_ptr,
                                      cache,
                                      occl_set,
                                      generate_options);

            return status;
        }
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

    pub trait LogHandler {
        fn handle_log_event(&mut self, msg: &str);
    }

    #[repr(C)]
    struct AbstractLogHandlerBinding<T> where T: LogHandler {
        handle_log_event: unsafe extern fn(*mut T, msg: *const ffi::c_char),
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
        fn ffi_add_log_handler(log_handler: *mut ffi::c_void);
        fn ffi_remove_log_handler(log_handler: *mut ffi::c_void);
    }

    pub fn add_log_handler<T>(log_handler: &mut Box<T>) where T: LogHandler {
        unsafe extern "C" fn handle_log_event<T>(context: *mut T, cmsg: *const ffi::c_char)
            where T: LogHandler
        {
            let handler_ref: &mut T = &mut *context;

            let msg = ffi::CStr::from_ptr(cmsg).to_str().unwrap();
            handler_ref.handle_log_event(msg);
        }

        let context: *mut T = log_handler.as_mut();
        let binding: Box<AbstractLogHandlerBinding<T>> = Box::new(AbstractLogHandlerBinding {
            handle_log_event,
            context,
        });

        let binding_ptr: *mut ffi::c_void = Box::into_raw(binding) as *mut ffi::c_void;
        unsafe {
            ffi_add_log_handler(binding_ptr);
        }
    }

    pub fn remove_log_handler<T>(log_handler: &mut Box<T>) where T: LogHandler {
        unsafe extern "C" fn handle_log_event<T>(_context: *mut T, _cmsg: *const ffi::c_char)
            where T: LogHandler
        {}

        let context = log_handler.as_mut() as *mut T;
        let binding: Box<AbstractLogHandlerBinding<T>> = Box::new(AbstractLogHandlerBinding {
            handle_log_event,
            context,
        });

        let binding_ptr: *mut ffi::c_void = Box::into_raw(binding) as *mut ffi::c_void;
        unsafe {
            ffi_remove_log_handler(binding_ptr);
        }
    }

    extern "C" {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt3logEPKwNS_8LogLevelE"]
        fn prt_log(msg: *const libc::wchar_t, level: LogLevel);
    }

    pub fn log(msg: &str, level: LogLevel) {
        let cs_vec = crate::helpers::from_string_to_wchar_vec(msg);
        unsafe {
            prt_log(cs_vec.as_ptr(), level);
        }
    }

    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    #[derive(PartialEq)]
    #[derive(Debug)]
    #[repr(C)]
    pub enum Status {
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

    extern "C" {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt20getStatusDescriptionENS_6StatusE"]
        fn ffi_get_status_description(input: Status) -> *const ffi::c_char;
    }

    pub fn get_status_description(status: Status) -> String {
        unsafe {
            let status_description_cchar_ptr = ffi_get_status_description(status);
            let status_description_cstr = ffi::CStr::from_ptr(status_description_cchar_ptr);
            let status_description = status_description_cstr.to_str().unwrap_or_default();
            return String::from(status_description);
        }
    }

    #[repr(C)]
    struct Version {
        version_major: i32,
        version_minor: i32,
        version_build: i32,

        name: *const ffi::c_char,
        full_name: *const ffi::c_char,
        version: *const ffi::c_char,
        build_config: *const ffi::c_char,
        build_os: *const ffi::c_char,
        build_arch: *const ffi::c_char,
        build_tc: *const ffi::c_char,
        build_date: *const ffi::c_char,

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
        cga_version: *const ffi::c_char,
        cga_version_w: *const i32,

        cgac_version_major: i32,
        cgac_version_minor: i32,
        cgac_version: *const ffi::c_char,
        cgac_version_w: *const i32,
    }

    extern "C" {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        #[link_name = "\u{1}_ZN3prt10getVersionEv"]
        fn ffi_get_version() -> *const Version;
    }

    pub struct PrtVersion {
        pub version_major: i32,
        pub version_minor: i32,
        pub version_build: i32,
        pub version_string: String,

        pub name: String,
        pub full_name: String,

        pub build_config: String,
        pub build_os: String,
        pub build_arch: String,
        pub build_tc: String,
        pub build_date: String,

        pub cga_version_major: i32,
        pub cga_version_minor: i32,
        pub cga_version_string: String,

        pub cgac_version_major: i32,
        pub cgac_version_minor: i32,
        pub cgac_version_string: String,
    }

    pub fn get_version() -> Result<PrtVersion, PrtError> {
        unsafe {
            let version_ptr = ffi_get_version();
            if version_ptr == ptr::null() {
                return Err(PrtError {
                    message: "Could not get PRT version info".to_string(),
                    status: None,
                });
            }
            let version_ref = &*version_ptr;
            let ver = PrtVersion {
                version_major: version_ref.version_major,
                version_minor: version_ref.version_minor,
                version_build: version_ref.version_build,
                version_string: crate::helpers::from_char_ptr_to_string(version_ref.version),
                name: crate::helpers::from_char_ptr_to_string(version_ref.name),
                full_name: crate::helpers::from_char_ptr_to_string(version_ref.full_name),
                build_config: crate::helpers::from_char_ptr_to_string(version_ref.build_config),
                build_os: crate::helpers::from_char_ptr_to_string(version_ref.build_os),
                build_arch: crate::helpers::from_char_ptr_to_string(version_ref.build_arch),
                build_tc: crate::helpers::from_char_ptr_to_string(version_ref.build_tc),
                build_date: crate::helpers::from_char_ptr_to_string(version_ref.build_date),
                cga_version_major: version_ref.cga_version_major,
                cga_version_minor: version_ref.cga_version_minor,
                cga_version_string: crate::helpers::from_char_ptr_to_string(version_ref.cga_version),
                cgac_version_major: version_ref.cgac_version_major,
                cgac_version_minor: version_ref.cgac_version_minor,
                cgac_version_string: crate::helpers::from_char_ptr_to_string(version_ref.cgac_version),
            };
            Ok(ver)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn assert_cstring(prefix: &str, raw_val: *const ffi::c_char, expected_val: &str) {
            let val = crate::helpers::from_char_ptr_to_string(raw_val);
            assert_string(prefix, val, expected_val);
        }

        fn assert_wcstring(prefix: &str, raw_val: *const libc::wchar_t, expected_val: &str) {
            let val = crate::helpers::from_wchar_ptr_to_string(raw_val);
            assert_string(prefix, val, expected_val);
        }

        fn assert_string(prefix: &str, raw_val: String, expected_val: &str) {
            assert_eq!(raw_val, expected_val, "{}: assertion error", prefix);
        }

        fn assert_int(prefix: &str, raw_val: i32, expected_val: i32) {
            assert_eq!(raw_val, expected_val, "{}: assertion error", prefix);
        }

        #[test]
        fn prt_get_version() {
            unsafe {
                let ver = &*ffi_get_version();
                assert_cstring("prt::Version::mName", ver.name, "ArcGIS Procedural Runtime");
                assert_cstring("prt::Version::mVersion", ver.version, "2.7.8538");
                assert_cstring("prt::Version::mBuildConfig", ver.build_config, "PRT_BC_REL");
                assert_cstring("prt::Version::mBuildOS", ver.build_os, "linux");
                assert_cstring("prt::Version::mBuildArch", ver.build_arch, "x86_64");
                assert_cstring("prt::Version::mBuildTC", ver.build_tc, "PRT_TC_GCC93");
                assert_cstring("prt::Version::mBuildDate", ver.build_date, "2022-10-04 15:48");

                assert_wcstring("prt::Version::mwName", ver.name_w, "ArcGIS Procedural Runtime");
                assert_wcstring("prt::Version::mwBuildDate", ver.build_date_w, "2022-10-04 15:48");

                assert_int("prt::Version::mCGAVersionMajor", ver.cga_version_major, 2022);
                assert_int("prt::Version::mCGAVersionMinor", ver.cga_version_minor, 1);

                assert_int("prt::Version::mCGACVersionMajor", ver.cgac_version_major, 1);
                assert_int("prt::Version::mCGACVersionMinor", ver.cgac_version_minor, 19);
            }
        }

        #[test]
        fn prt_get_status_description() {
            unsafe {
                let status_description_cchar_ptr = ffi_get_status_description(Status::STATUS_OUT_OF_MEM);
                let status_description = crate::helpers::from_char_ptr_to_string(status_description_cchar_ptr);
                assert_eq!(status_description, "Out of memory.");
            }
        }

        #[test]
        fn create_attribute_map() {
            let mut map = collections::HashMap::new();
            map.insert("foo".to_string(), PrimitiveType::String("bar".to_string()));
            assert_eq!(map.get("foo"), Some(&PrimitiveType::String("bar".to_string())));
        }
    }
}

mod helpers {
    use std::{ffi, fs, path};

    pub(crate) fn from_char_ptr_to_string(cchar_ptr: *const ffi::c_char) -> String {
        let val_cstr = unsafe { ffi::CStr::from_ptr(cchar_ptr) };
        return val_cstr.to_str().unwrap_or_default().to_string();
    }

    #[allow(dead_code)]
    #[cfg(target_os = "linux")]
    fn wchar_is_utf32() -> bool {
        true
    }

    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    fn wchar_is_utf32() -> bool {
        false
    }

    #[allow(dead_code)]
    pub fn from_wchar_ptr_to_string(ptr: *const libc::wchar_t) -> String {
        assert!(!ptr.is_null());
        let ptr_len = unsafe { libc::wcslen(ptr) };
        assert!(ptr_len > 0);
        return if wchar_is_utf32() {
            let widestring_result = unsafe { widestring::U32CString::from_ptr(ptr as *const u32, ptr_len) };
            let cstring = widestring_result.expect("could not convert wchar_t array to UTF32 string");
            cstring.to_string().expect("could not convert to Rust string")
        } else {
            let widestring_result = unsafe { widestring::U16CString::from_ptr(ptr as *const u16, ptr_len) };
            let cstring = widestring_result.expect("could not convert wchar_t array to UTF16 string");
            cstring.to_string().expect("could not convert to Rust string")
        };
    }

    pub fn from_string_to_wchar_vec(msg: &str) -> Vec<libc::wchar_t> {
        return if cfg!(linux) {
            let wide_msg = widestring::U32CString::from_str(msg)
                .expect("cannot convert to UTF-32/wchar_t");
            wide_msg.into_vec_with_nul().iter().map(|&e| e as libc::wchar_t).collect()
        } else {
            let wide_msg = widestring::U16CString::from_str(msg)
                .expect("cannot convert to UTF-16/wchar_t");
            wide_msg.into_vec_with_nul().iter().map(|&e| e as libc::wchar_t).collect()
        };
    }

    // TODO: deduplicate with get_dependencies_path in build.rs
    pub fn get_cesdk_path() -> path::PathBuf {
        let out_dir = env!("OUT_DIR");
        let crate_root = path::PathBuf::from(out_dir);
        let deps_path = crate_root.join("prust_custom_deps");
        let cesdk_path_entry = fs::read_dir(deps_path)
            .expect("cannot read deps dir")
            .filter(|p| p.is_ok() && p.as_ref().unwrap().file_type().unwrap().is_dir())
            .find(|p| p.as_ref().unwrap()
                .path().file_name().unwrap()
                .to_str().unwrap().starts_with("esri_ce_sdk-"));
        return cesdk_path_entry.expect("could not find cesdk lib dir").unwrap().path();
    }
}
