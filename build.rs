use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::process::Command;

fn main(){
    find_llvm_lib().map_or_else(|| {
        println!("cargo:rustc-cfg=use_extern");
    }, |lib| {
        let lname = path_to_link_name(&lib).expect("could not generate argument to -l");
        println!("cargo:rustc-link-lib=dylib={}", lname);
    });
}

fn path_to_link_name<'a, P: AsRef<Path>>(p: &'a P) -> Option<&'a str> {
    p.as_ref().file_stem().and_then(OsStr::to_str).map(|s| {
        s.trim_left_matches("lib")
    })
}

fn find_llvm_lib() -> Option<PathBuf> {
    // TODO: cargo does not export us a $RUSTC
    let sysroot = Command::new("rustc").arg("--print=sysroot").output()
        .expect("could not execute rustc");
    // FIXME: may be not utf8?
    let mut path = String::from_utf8(sysroot.stdout).expect("rustc output not utf-8");
    while let Some(ch) = path.pop() {
        if !ch.is_whitespace() {
            path.push(ch);
            break;
        }
    }
    println!("sysroot is {:?}", path);
    let path = PathBuf::from(path);
    for entry in path.join("lib").read_dir().expect("could not read dir") {
        let entry_path = entry.expect("could not read dir").path();
        let ret = if let Some(Some(st)) = entry_path.file_name().map(|f| f.to_str()) {
            println!("{:?} in lib", st);
            st.contains("rustc_llvm")
        } else { false };
        if ret {
            return Some(entry_path);
        }
    }
    // Do not search for windows stuff yet, it simply does not work.
    // for entry in path.join("bin").read_dir().expect("could not read dir") {
    //     let entry_path = entry.expect("could not read dir").path();
    //     let ret = if let Some(Some(st)) = entry_path.file_name().map(|f| f.to_str()) {
    //         println!("{:?} in bin", st);
    //         st.contains(".dll.lib") && st.contains("rustc_llvm")
    //     } else { false };
    //     if ret {
    //         return Some(entry_path);
    //     }
    // }
    None
}
