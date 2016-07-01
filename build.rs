use std::path::PathBuf;
use std::process::Command;

fn main(){
    let (path, lib) = find_llvm_lib();
    println!("cargo:rustc-link-search=native={}", path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);
}

fn find_llvm_lib() -> (PathBuf, String) {
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
        let entry_path = entry_path.with_extension("");
        let fname = entry_path.file_name();
        if let Some(Some(st)) = fname.map(|f| f.to_str()) {
            println!("{:?} in lib", st);
            if let Some(i) = st.find("rustc_llvm") {
                return (path, String::from(&st[i..]));
            }
        }
    }
    for entry in path.join("bin").read_dir().expect("could not read dir") {
        let entry_path = entry.expect("could not read dir").path();
        let entry_path = entry_path.with_extension("");
        let fname = entry_path.file_name();
        if let Some(Some(st)) = fname.map(|f| f.to_str()) {
            println!("{:?} in bin", st);
            if let Some(i) = st.find("rustc_llvm") {
                return (path, String::from(&st[i..]));
            }
        }
    }
    panic!("could not find rustc_llvm library");

}
