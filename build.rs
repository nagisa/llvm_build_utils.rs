#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate kernel32;


use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::process::Command;

fn main(){
    let lib = find_llvm_lib();
    {
    let lname = path_to_link_name(&lib).expect("could not generate argument to -l");
    println!("cargo:rustc-link-lib=dylib={}", lname);
    }
    hack::that_windows_hack(lib);
}

fn path_to_link_name<'a, P: AsRef<Path>>(p: &'a P) -> Option<&'a str> {
    p.as_ref().file_stem().and_then(OsStr::to_str).map(|s| {
        s.trim_left_matches("lib")
    })
}

fn find_llvm_lib() -> PathBuf {
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
            return entry_path;
        }
    }
    for entry in path.join("bin").read_dir().expect("could not read dir") {
        let entry_path = entry.expect("could not read dir").path();
        let ret = if let Some(Some(st)) = entry_path.file_name().map(|f| f.to_str()) {
            println!("{:?} in bin", st);
            st.contains(".dll.lib") && st.contains("rustc_llvm")
        } else { false };
        if ret {
            return entry_path;
        }
    }
    panic!("could not find rustc_llvm library");
}

#[cfg(not(windows))]
mod hack {
    use std::path::PathBuf;
    pub fn that_windows_hack(_: PathBuf) {}
}

#[cfg(windows)]
mod hack {
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;
    use std::ffi::{CStr, CString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use super::winapi;
    use super::kernel32;
    struct IMAGE_EXPORT_DIRECTORY {
        Characteristics:       winapi::DWORD,
        TimeDateStamp:         winapi::DWORD,
        MajorVersion:          winapi::WORD,
        MinorVersion:          winapi::WORD,
        Name:                  winapi::DWORD,
        Base:                  winapi::DWORD,
        NumberOfFunctions:     winapi::DWORD,
        NumberOfNames:         winapi::DWORD,
        AddressOfFunctions:    winapi::DWORD,
        AddressOfNames:        winapi::DWORD,
        AddressOfNameOrdinals: winapi::DWORD,
    }
    const IMAGE_DOS_SIGNATURE: winapi::WORD = 0x5A4D;
    const DONT_RESOLVE_DLL_REFERENCES: winapi::DWORD = 0x1;
    const IMAGE_NT_SIGNATURE: winapi::DWORD = 0x00004550;

    // At this point I’m wondering how much farther does this hack of a library needs to go. On one
    // hand, I’m very angry at windows for not exposing public LLVM symbols through the library
    // which wraps it, on another I’m pretty sure I shouldn’t go ahead, read down the dll down
    // myself and figure out the name of symbol for a function which rustc_llvm defines to
    // initialize the LLVM…
    pub fn that_windows_hack(libpath: PathBuf) {
        /* very */ unsafe { // stuff
            let p = libpath.with_extension("").into_os_string();
            println!("loading {:?}", p);
            let wide_filename: Vec<u16> = p.encode_wide().chain(Some(0)).collect();
            let lib = kernel32::LoadLibraryExW(wide_filename.as_ptr(), ::std::ptr::null_mut(),
                                               DONT_RESOLVE_DLL_REFERENCES);
            assert!(!lib.is_null(), "could not load teh library");
            println!("loaded indeed {:p}", lib);
            assert_eq!(*(lib as winapi::PWORD), IMAGE_DOS_SIGNATURE);
            println!("DOS indeed");
            let nthdr_off = (*((lib as winapi::PBYTE).offset(0x3c)
                             as *mut winapi::DWORD)) as isize;
            println!("nthdr off {:x}", nthdr_off);
            let ref nthdr = *((lib as winapi::PBYTE).offset(nthdr_off)
                               as winapi::PIMAGE_NT_HEADERS);
            assert_eq!(nthdr.Signature, IMAGE_NT_SIGNATURE);
            println!("NT indeed");
            assert!(nthdr.OptionalHeader.NumberOfRvaAndSizes > 0);
            println!("Has that many things {:?}", nthdr.OptionalHeader.NumberOfRvaAndSizes);
            let exports_off = nthdr.OptionalHeader
                                   .DataDirectory[winapi::IMAGE_DIRECTORY_ENTRY_EXPORT as usize]
                                   .VirtualAddress as isize;
            let ref exports = *((lib as winapi::PBYTE).offset(exports_off)
                            as *mut IMAGE_EXPORT_DIRECTORY);
            assert!(exports.AddressOfNames != 0);
            println!("{} exports @ {:?}", exports.NumberOfNames, exports.AddressOfNames);
            let names = (lib as winapi::PBYTE).offset(exports.AddressOfNames as isize)
                         as *mut winapi::DWORD;
            for i in 0..exports.NumberOfNames {
                let name = (lib as winapi::PBYTE).offset(*names.offset(i as isize) as isize)
                            as *mut _;
                let symbol = CStr::from_ptr(name).to_string_lossy();
                println!("symbol name {} at {:?}", i, symbol);
                if !symbol.contains("initialize_available_targets") { continue }
                println!("found symbol {}", symbol);
                let mut f = File::create("src/that_windows_hack.rs")
                    .expect("could not open the hack");
                writeln!(f, r#"extern "Rust" {{ fn {}(); }}"#, symbol).unwrap();
                writeln!(f, r#"pub unsafe fn init_llvm() {{ {}() }}"#, symbol).unwrap();
                println!("cargo:rustc-cfg=do_windows_hack");
                return;
            }
        }
    }
}
