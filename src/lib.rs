use std::ffi::c_char;
use libc;

#[repr(C)]
pub struct Version {
    pub version_major: i32,
    pub version_minor: i32,
    pub version_build: i32,

    pub name: *const c_char,
    pub full_name: *const c_char,
    pub version: *const c_char,
    pub build_config: *const c_char,
    pub build_os: *const c_char,
    pub build_arch: *const c_char,
    pub build_tc: *const c_char,
    pub build_date: *const c_char,

    pub name_w: *const libc::wchar_t,
    pub full_name_w: *const libc::wchar_t,
    pub version_w: *const libc::wchar_t,
    pub build_config_w: *const libc::wchar_t,
    pub build_os_w: *const libc::wchar_t,
    pub build_arch_w: *const libc::wchar_t,
    pub build_tc_w: *const libc::wchar_t,
    pub build_date_w: *const libc::wchar_t,

    pub cga_version_major: i32,
    pub cga_version_minor: i32,
    pub cga_version: *const c_char,
    pub cga_version_w: *const i32,

    pub cgac_version_major: i32,
    pub cgac_version_minor: i32,
    pub cgac_version: *const c_char,
    pub cgac_version_w: *const i32,
}

extern "C" {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[link_name = "\u{1}_ZN3prt10getVersionEv"]
    fn get_version() -> *const Version;

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    #[link_name = "\u{1}_ZN3prt20getStatusDescriptionENS_6StatusE"]
    fn get_status_description(input: i32) -> *const c_char;
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::slice;
    use super::*;

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
}
