use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn get_out_path() -> PathBuf {
    return PathBuf::from(env::var("OUT_DIR").expect("cannot get OUT_DIR env var"));
}

fn get_dependencies_path() -> PathBuf { // TODO: make static
    let root_path = get_out_path();
    let deps = root_path.join("prust_custom_deps");
    if !deps.exists() {
        std::fs::create_dir(deps.as_path()).expect("could not create deps directory");
    }
    return deps;
}

// TODO: can we use the cargo dependency tools for this instead of re-inventing the wheel?
fn download(my_url: &url::Url) -> PathBuf {
    let url_path = PathBuf::from(my_url.path());
    let local_file_name = url_path.file_name().unwrap();

    let deps_path = get_dependencies_path();
    let local_file_path = deps_path.join(local_file_name);
    let extraction_path = deps_path.join(local_file_path.file_stem().unwrap());
    if local_file_path.exists() { // TODO: allow re-download with 'force' param
        return extraction_path;
    }

    let response = reqwest::blocking::get(my_url.to_string()).unwrap();
    let content = response.bytes().unwrap();
    let mut local_file = std::fs::File::create(&local_file_path).unwrap();
    let _res = std::io::copy(&mut content.as_ref(), &mut local_file);

    let local_archive = std::fs::File::open(&local_file_path).unwrap();
    let _extract_result = zip_extract::extract(local_archive, &extraction_path, false);

    return extraction_path;
}

fn main() {
    let cesdk_archive_url = url::Url::parse("https://github.com/Esri/cityengine-sdk/releases/download/2.7.8538/esri_ce_sdk-2.7.8538-rhel7-gcc93-x86_64-rel-opt.zip");
    let cesdk_root_path = download(&cesdk_archive_url.unwrap());
    let cesdk_bin_path = cesdk_root_path.join("bin");

    // patching rpath in cesdk on linux so core finds glutess
    let output = Command::new("patchelf")
        .arg("--set-rpath")
        .arg("$ORIGIN")
        .arg(format!("{}/libcom.esri.prt.core.so", cesdk_bin_path.to_str().unwrap()))
        .output()
        .expect("failed to run patchelf");
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    println!("cargo:rustc-link-search=native={}", cesdk_bin_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=com.esri.prt.core");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", cesdk_bin_path.to_str().unwrap());

    let cesdk_cmake_path = cesdk_root_path.join("cmake");
    let dst = cmake::Config::new("cpp")
        .define("prt_DIR", cesdk_cmake_path)
        .build();
    let bindings_lib_path = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", bindings_lib_path.display());
    println!("cargo:rustc-link-lib=static=bindings");
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rerun-if-changed=cpp/bindings.cpp");
    println!("cargo:rerun-if-changed=cpp/bindings.h");
}
