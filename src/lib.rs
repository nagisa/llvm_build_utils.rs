//! build.rs utils for building LLVM-IR and bytecode
//!
//! Building assembly in a cross-os manner is a very painful endeavour requiring to be aware of
//! various assemblers available for each platform etc. Using LLVM-IR as *the* assembly is the
//! better alternative, and this crate provides tools converting your .ll or .bc files into .a you
//! can link into your next great thing.
//!
//! Uses the same LLVM as rustc thus avoiding the need for installed LLVM.
//!
//! While this library does work on stable, it may break between Rust releases due to abuse of
//! stability system.
#![allow(non_camel_case_types, non_upper_case_globals)]
extern crate libc;
extern crate mktemp;

use std::path::Path;
use std::ffi::{CString, CStr};
use std::sync::{Once, ONCE_INIT};

type LLVMBool = libc::c_uint;
const LLVMTrue: LLVMBool = 1;
const LLVMFalse: LLVMBool = 0;
pub enum LLVMPassRegistry_opaque {}
pub type LLVMPassRegistryRef = *mut LLVMPassRegistry_opaque;
#[allow(missing_copy_implementations)]
enum LLVMContext_opaque {}
type LLVMContextRef = *mut LLVMContext_opaque;
#[allow(missing_copy_implementations)]
enum LLVMMemoryBuffer_opaque {}
type LLVMMemoryBufferRef = *mut LLVMMemoryBuffer_opaque;
#[allow(missing_copy_implementations)]
enum LLVMModule_opaque {}
type LLVMModuleRef = *mut LLVMModule_opaque;
#[allow(missing_copy_implementations)]
pub enum LLVMTarget_opaque {}
pub type LLVMTargetRef = *mut LLVMTarget_opaque;
pub enum LLVMTargetMachine_opaque {}
pub type LLVMTargetMachineRef = *mut LLVMTargetMachine_opaque;
pub enum LLVMArchive_opaque {}
pub type LLVMArchiveRef = *mut LLVMArchive_opaque;
pub enum LLVMArchiveIterator_opaque {}
pub type LLVMArchiveIteratorRef = *mut LLVMArchiveIterator_opaque;
pub enum LLVMArchiveChild_opaque {}
pub type LLVMArchiveChildRef = *mut LLVMArchiveChild_opaque;
#[allow(missing_copy_implementations)]
pub enum LLVMRustArchiveMember_opaque {}
pub type LLVMRustArchiveMemberRef = *mut LLVMRustArchiveMember_opaque;



#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum RelocMode {
    Default = 0,
    Static = 1,
    PIC = 2,
    DynamicNoPic = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CodeGenModel {
    Default = 0,
    JITDefault = 1,
    Small = 2,
    Kernel = 3,
    Medium = 4,
    Large = 5,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum CodeGenOptLevel {
    O0 = 0,
    O1 = 1,
    O2 = 2,
    O3 = 3,
}

#[allow(dead_code)]
#[repr(C)]
enum  VerifierFailureAction {
    AbortProcess = 0,
    PrintMessage = 1,
    ReturnStatus = 2,
}

#[allow(dead_code)]
#[repr(C)]
enum CodeGenFileType {
    Assembly = 0,
    Object = 1,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum ArchiveKind {
    K_GNU,
    K_MIPS64,
    K_BSD,
    K_COFF,
}

extern {
    fn LLVMContextCreate() -> LLVMContextRef;
    fn LLVMContextDispose(C: LLVMContextRef);
    fn LLVMParseIRInContext(context: LLVMContextRef,
                            buf: LLVMMemoryBufferRef,
                            om: *mut LLVMModuleRef,
                            msg: *mut *mut libc::c_char) -> LLVMBool;
    // fn LLVMDisposeMemoryBuffer(MemBuf: LLVMMemoryBufferRef);
    fn LLVMSetTarget(M: LLVMModuleRef, Triple: *const libc::c_char);
    fn LLVMDisposeModule(M: LLVMModuleRef);
    fn LLVMVerifyModule(_: LLVMModuleRef, _: VerifierFailureAction, _: *mut *mut libc::c_char)
    -> LLVMBool;
    fn LLVMDisposeMessage(_: *mut libc::c_char);
    fn LLVMCreateTargetMachine(tr: LLVMTargetRef,
                               triple: *const libc::c_char,
                               cpu: *const libc::c_char,
                               features: *const libc::c_char,
                               lvl: CodeGenOptLevel,
                               reloc: RelocMode,
                               cm: CodeGenModel) -> LLVMTargetMachineRef;
    fn LLVMDisposeTargetMachine(_: LLVMTargetMachineRef);
    fn LLVMTargetMachineEmitToFile (_: LLVMTargetMachineRef,
                                            _: LLVMModuleRef,
                                            filename: *const libc::c_char,
                                            _: CodeGenFileType,
                                            err: *mut *mut libc::c_char) -> LLVMBool;
    fn LLVMGetTargetFromTriple(triple: *const libc::c_char,
                               _: *mut LLVMTargetRef,
                               err: *mut *mut libc::c_char) -> LLVMBool;

    // Unstable Rust’s LLVM bindings
    fn LLVMRustCreateMemoryBufferWithContentsOfFile(Path: *const libc::c_char)
                                                    -> LLVMMemoryBufferRef;
    fn LLVMRustGetLastError() -> *const libc::c_char;
    fn LLVMRustArchiveMemberNew(_: *const libc::c_char,
                                _: *const libc::c_char,
                                _: LLVMArchiveChildRef) -> LLVMRustArchiveMemberRef;
    fn LLVMRustArchiveMemberFree(_: LLVMRustArchiveMemberRef);
    fn LLVMRustWriteArchive(Dst: *const libc::c_char,
                            NumMembers: libc::size_t,
                            Members: *const LLVMRustArchiveMemberRef,
                            WriteSymbtab: bool,
                            Kind: ArchiveKind) -> libc::c_int;
}

#[derive(Debug)]
pub struct BuildOptions {
    pub triple: String,
    pub cpu: String,
    pub attr: String,
    pub model: CodeGenModel,
    pub reloc: RelocMode,
    pub opt: CodeGenOptLevel,
    pub ar_section_name: String,
}

impl Default for BuildOptions {
    fn default() -> BuildOptions {
        BuildOptions {
            triple: ::std::env::var("TARGET").unwrap_or(String::new()),
            cpu: String::new(),
            attr: String::new(),
            model: CodeGenModel::Default,
            reloc: RelocMode::Default,
            opt: CodeGenOptLevel::O0,
            ar_section_name: String::new(),
        }
    }
}

fn initialize_llvm() {
    static ONCE: Once = ONCE_INIT;

    macro_rules! init_target(
        ($($method:ident),*) => { {
            extern { $(fn $method();)* }
            $($method();)*
        } }
    );

    ONCE.call_once(|| unsafe {
        init_target!(LLVMInitializeX86TargetInfo,
                     LLVMInitializeX86Target,
                     LLVMInitializeX86TargetMC,
                     LLVMInitializeX86AsmPrinter,
                     LLVMInitializeX86AsmParser,
                     LLVMInitializeARMTargetInfo,
                     LLVMInitializeARMTarget,
                     LLVMInitializeARMTargetMC,
                     LLVMInitializeARMAsmPrinter,
                     LLVMInitializeARMAsmParser,
                     LLVMInitializeAArch64TargetInfo,
                     LLVMInitializeAArch64Target,
                     LLVMInitializeAArch64TargetMC,
                     LLVMInitializeAArch64AsmPrinter,
                     LLVMInitializeAArch64AsmParser,
                     LLVMInitializeMipsTargetInfo,
                     LLVMInitializeMipsTarget,
                     LLVMInitializeMipsTargetMC,
                     LLVMInitializeMipsAsmPrinter,
                     LLVMInitializeMipsAsmParser,
                     LLVMInitializePowerPCTargetInfo,
                     LLVMInitializePowerPCTarget,
                     LLVMInitializePowerPCTargetMC,
                     LLVMInitializePowerPCAsmPrinter,
                     LLVMInitializePowerPCAsmParser);
    });
}

macro_rules! fail_if {
    ($ex: expr, $($args: tt)*) => {
        if $ex { return Err(format!($($args)*)) }
    }
}

/// Produce a static library containing machine code for LLVM-IR/bytecode files at pathes produced
/// by the iterator.
pub fn build_archive<'a, P: 'a, I>(iter: I, archive: P)
-> Result<(), String>
where P: AsRef<Path>, I: IntoIterator<Item=&'a (P, BuildOptions)>
{
    initialize_llvm();
    let mut strings = vec![];
    let mut members = vec![];
    let mut temps = vec![];
    unsafe {
        let ctx = LLVMContextCreate();
        fail_if!(ctx.is_null(), "could not create the context");
        for &(ref p, ref opt) in iter {
            let mut module = ::std::ptr::null_mut();
            let mut msg = ::std::ptr::null_mut();

            // Read the LLVM-IR/BC into memory
            let path = try!(CString::new(
                try!(p.as_ref().to_str().ok_or_else(|| String::from("input filename is not utf-8")))
            ).map_err(|_| String::from("input filename contains nulls")));
            let buf = LLVMRustCreateMemoryBufferWithContentsOfFile(path.as_ptr());
            fail_if!(buf.is_null(), "could not open input file {:?}: {:?}",
                    p.as_ref(), CStr::from_ptr(LLVMRustGetLastError()));

            // Parse the IR/BC
            LLVMParseIRInContext(ctx, buf, &mut module, &mut msg);
            fail_if!(module.is_null(), "module could not be parsed successfully: {:?}",
                     CStr::from_ptr(msg));
            if LLVMVerifyModule(module, VerifierFailureAction::ReturnStatus, &mut msg) == LLVMTrue {
                let ret = Err(format!("Module is not valid: {:?}", CStr::from_ptr(msg)));
                LLVMDisposeMessage(msg);
                return ret;
            }

            // Build the IR/BC to object file
            let triple = CString::new(opt.triple.clone()).expect("triple contains null bytes");
            let cpu = CString::new(opt.cpu.clone()).expect("cpu contains null bytes");
            let attr = CString::new(opt.attr.clone()).expect("attr contains null bytes");
            if !opt.triple.is_empty() {
                LLVMSetTarget(module, triple.as_ptr());
            }
            let mut target = ::std::ptr::null_mut();
            let status = LLVMGetTargetFromTriple(triple.as_ptr(), &mut target, &mut msg);
            fail_if!(status != LLVMFalse, "could not generate target from triple {}: {:?}",
                     opt.triple, CStr::from_ptr(msg));
            let machine = LLVMCreateTargetMachine(target,
                                                  triple.as_ptr(),
                                                  cpu.as_ptr(),
                                                  attr.as_ptr(),
                                                  opt.opt,
                                                  opt.reloc,
                                                  opt.model);
            fail_if!(machine.is_null(), "could not create the target machine \
                                         (likely invalid BuildOptions {:?})", opt);


            let tmp = try!(mktemp::Temp::new_file_in(
                p.as_ref().parent().expect("cannot get basename of input filename")
            ).map_err(|e| format!("could not create temp file: {}", e)));
            let object_file = try!(CString::new(
                try!(tmp.as_ref().to_str().ok_or_else(|| String::from("object path is not utf-8")))
            ).map_err(|_| String::from("object filename contains nulls")));
            temps.push(tmp);

            let status = LLVMTargetMachineEmitToFile(machine,
                                                     module,
                                                     object_file.as_ptr(),
                                                     CodeGenFileType::Object,
                                                     &mut msg);
            fail_if!(status == LLVMTrue, "could not generate object file: {:?}",
                    CStr::from_ptr(msg));

            // Put the built objects into an archive
            let name = try!(CString::new(opt.ar_section_name.clone())
                            .map_err(|_| String::from("archive member name contains nulls")));
            members.push(LLVMRustArchiveMemberNew(object_file.as_ptr(),
                                                  name.as_ptr(),
                                                  std::ptr::null_mut()));
            strings.push(name);
            strings.push(object_file);

            LLVMDisposeTargetMachine(machine);
            LLVMDisposeModule(module);
            // FIXME: SIGSEGVS for some reason
            // LLVMDisposeMemoryBuffer(buf);
        }
        let dest = try!(CString::new(try!(
            archive.as_ref().to_str().ok_or_else(|| String::from("archive filename is not utf-8"))
        )).map_err(|_| String::from("output file has interior nulls")));
        let r = LLVMRustWriteArchive(dest.as_ptr(),
                                     members.len() as libc::size_t,
                                     members.as_ptr(),
                                     true,
                                     ArchiveKind::K_GNU);
        fail_if!(r != 0, "{:?}", {
            let err = LLVMRustGetLastError();
            if err.is_null() {
                "failed to write archive".to_string()
            } else {
                String::from_utf8_lossy(CStr::from_ptr(err).to_bytes())
                        .into_owned()
            }
        });
        for member in members {
            LLVMRustArchiveMemberFree(member);
        }
        LLVMContextDispose(ctx);
        Ok(())
    }
}
