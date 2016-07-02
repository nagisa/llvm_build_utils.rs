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
            !st.contains(".dll.lib") && st.contains("rustc_llvm")
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
    use std::ffi::{CStr, CString};
    use super::winapi;
    use super::kernel32;
    struct IMAGE_DOS_HEADER {
        e_magic:    winapi::WORD,
        e_cblp:     winapi::WORD,
        e_cp:       winapi::WORD,
        e_crlc:     winapi::WORD,
        e_cparhdr:  winapi::WORD,
        e_minalloc: winapi::WORD,
        e_maxalloc: winapi::WORD,
        e_ss:       winapi::WORD,
        e_sp:       winapi::WORD,
        e_csum:     winapi::WORD,
        e_ip:       winapi::WORD,
        e_cs:       winapi::WORD,
        e_lfarlc:   winapi::WORD,
        e_ovno:     winapi::WORD,
        e_res:      [winapi::WORD; 4],
        e_oemid:    winapi::WORD,
        e_oeminfo:  winapi::WORD,
        e_res2:     [winapi::WORD; 10],
        e_lfanew:   winapi::LONG,
    }

    struct IMAGE_EXPORT_DIRECTORY {
        Characteristics:       winapi::DWORD,
        TimeDateStamp:         winapi::DWORD,
        MajorVersion:          winapi::WORD,
        MinorVersion:          winapi::WORD,
        Name:                  winapi::DWORD,
        Base:                  winapi::DWORD,
        NumberOfFunctions:     winapi::DWORD,
        NumberOfNames:         winapi::DWORD,
        AddressOfFunctions:    *mut winapi::LPDWORD,
        AddressOfNames:        *mut winapi::LPDWORD,
        AddressOfNameOrdinals: *mut winapi::LPWORD,
    }
    const IMAGE_DOS_SIGNATURE: winapi::WORD = 0x5A4D;
    const LOAD_LIBRARY_AS_DATAFILE: winapi::DWORD = 0x2;
    const IMAGE_NT_SIGNATURE: winapi::DWORD = 0x00004550;

    // At this point I’m wondering how much farther does this hack of a library needs to go. On one
    // hand, I’m very angry at windows for not exposing public LLVM symbols through the library
    // which wraps it, on another I’m pretty sure I shouldn’t go ahead, read down the dll down
    // myself and figure out the name of symbol for a function which rustc_llvm defines to
    // initialize the LLVM…
    pub fn that_windows_hack(libpath: PathBuf) {
        /* very */ unsafe { // stuff
            let p = libpath.with_extension("").into_os_string().into_string().ok().unwrap();
            let cstr = CString::new(p).expect("null bytes");
            let lib = kernel32::LoadLibraryExW(cstr.as_ptr(), ::std::ptr::null(),
                                               LOAD_LIBRARY_AS_DATAFILE);
            assert!(lib.is_null(), "could not load teh library");
            let dos_hdr = *(lib as *mut IMAGE_DOS_HEADER);
            assert!(dos_hdr.e_magic == IMAGE_DOS_SIGNATURE);
            let nthdr = (lib as winapi::PBYTE).offset(dos_hdr.e_lfanew as isize)
                as winapi::PIMAGE_NT_HEADERS;
            assert!((*nthdr).Signature == IMAGE_NT_SIGNATURE);
            assert!((*nthdr).OptionalHeader.NumberOfRvaAndSizes > 0);
            let exports = (lib as winapi::PBYTE)
                .offset((*nthdr).OptionalHeader
                                .DataDirectory[winapi::IMAGE_DIRECTORY_ENTRY_EXPORT as usize]
                                .VirtualAddress as isize)
                as *mut IMAGE_EXPORT_DIRECTORY;
            assert!(!(*exports).AddressOfNames.is_null());
            let names = lib.offset((*exports).AddressOfNames) as *mut winapi::PBYTE;
            for i in 0..(*exports.NumberOfNames) {
                println!("{:?}", CStr::from_ptr(names.offset(i)));
            }
            panic!()
        }
    }
}
