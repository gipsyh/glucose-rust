use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:include=glucose/build");
    println!("cargo:rustc-link-search=native=glucose/build");
    println!("cargo:rustc-link-lib=glucose");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    if Path::new(".git").is_dir() {
        Command::new("git")
            .args(["submodule", "update", "--remote"])
            .status()
            .expect("Failed to update submodules.");
    } else {
        assert!(Path::new("glucose").is_dir(), "glucose source not included");
    }

    Command::new("mkdir")
        .current_dir("./glucose/")
        .args(["-p", "build"])
        .status()
        .expect("Failed to mkdir build dir");

    println!("cargo:rerun-if-changed=glucose/sources");

    Command::new("cmake")
        .current_dir("./glucose/build")
        .args(["../sources", "-O", "."])
        .status()
        .expect("Failed to build glucose using cmake");

    Command::new("make")
        .current_dir("./glucose/build")
        .args(["-j"])
        .status()
        .expect("Failed to build glucose using make");

    let bindings = bindgen::Builder::default()
        .header("glucose/sources/simp/interface.h")
        .clang_arg("-Iglucose/sources/")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++14")
        .vtable_generation(false)
        .allowlist_file("glucose/sources/simp/interface.h")
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings");
}
