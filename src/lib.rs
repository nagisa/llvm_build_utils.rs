//! build.rs utils for building LLVM-IR and bytecode
//!
//! Building assembly in a cross platform manner is a very painful endeavour requiring to be aware
//! of various assemblers available and syntaxes they use. Using LLVM-IR as *the* assembly is a
//! better alternative and this crate provides tools converting your `.ll` or `.bc` files into `.a`
//! archives containing machine code. These archives can then be statically linked to your project.
//!
//! This library does not need an installation of LLVM or `ar` to work. The LLVM which comes with
//! rustc is used instead. While this library works perfectly on stable versions of the compiler,
//! the library may be incompatible with a future version of rustc.
//!
//! # Usage
//!
//! First, you’ll want to both add a build script for your crate (`build.rs`) and also add this
//! crate to your `Cargo.toml` via:
//!
//! ```toml
//! [package]
//! # ...
//! build = "build.rs"
//!
//! [build-dependencies]
//! llvm_build_utils = "0.1"
//! ```
//!
//! Then write your `build.rs` like this:
//!
//! ```rust,no_run
//! extern crate llvm_build_utils;
//! use llvm_build_utils::*;
//!
//! fn main() {
//!     build_archive("libyourthing.a", &[
//!         ("input.ll", BuildOptions {
//!             ..BuildOptions::default() // customise how the file is built
//!         })
//!     ]).expect("error happened");
//!     // ...
//! }
//! ```
//!
//! Running a `cargo build` should produce `libyourthing.a` which then may be linked to your Rust
//! executable/library.
#![allow(non_camel_case_types, non_upper_case_globals)]
extern crate libc;
extern crate mktemp;

use std::path::Path;
use std::ffi::{CString, CStr};
use std::sync::{Once, ONCE_INIT};

type LLVMBool = libc::c_uint;
const LLVMTrue: LLVMBool = 1;
const LLVMFalse: LLVMBool = 0;
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
enum LLVMTarget_opaque {}
type LLVMTargetRef = *mut LLVMTarget_opaque;
enum LLVMTargetMachine_opaque {}
type LLVMTargetMachineRef = *mut LLVMTargetMachine_opaque;
enum LLVMArchiveChild_opaque {}
type LLVMArchiveChildRef = *mut LLVMArchiveChild_opaque;
#[allow(missing_copy_implementations)]
enum LLVMRustArchiveMember_opaque {}
type LLVMRustArchiveMemberRef = *mut LLVMRustArchiveMember_opaque;

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
                               lvl: Optimisation,
                               reloc: Relocations,
                               cm: CodegenModel) -> LLVMTargetMachineRef;
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

#[allow(dead_code)]
#[repr(C)]
enum VerifierFailureAction {
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


/// Relocation mode
///
/// This option decides how relocations are handled.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum Relocations {
    /// Target default relocation model
    Default = 0,
    /// Non-relocatable code
    Static = 1,
    /// Fully relocatable, position independent code
    PIC = 2,
    /// Relocatable external references, non-relocatable code
    DynamicNoPic = 3,
}

/// Codegen model
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CodegenModel {
    /// Target default code model
    Default = 0,
    /// Small code model
    Small = 2,
    /// Kernel code model
    Kernel = 3,
    /// Medium code model
    Medium = 4,
    /// Large code model
    Large = 5,
}

/// Codegen optimisation level
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub enum Optimisation {
    /// No codegen optimisation
    ///
    /// Corresponds to the -O0 option of `llc`
    O0 = 0,
    /// Some codegen optimisations
    ///
    /// Corresponds to the -O1 option of `llc`
    O1 = 1,
    /// Considerable codegen optimisations
    ///
    /// Corresponds to the -O2 option of `llc`
    O2 = 2,
    /// Heavy codegen optimisations
    ///
    /// Corresponds to the -O3 option of `llc`
    O3 = 3,
}

/// The format of generated archive file
#[allow(dead_code)]
#[repr(C)]
#[derive(Copy, Clone)]
enum ArchiveKind {
    Gnu,
    Mips64,
    Bsd,
    Coff,
}

#[derive(Debug)]
pub struct BuildOptions {
    /// Target triple to generate machine code for
    ///
    /// The target triple has the general format `<arch><sub>-<vendor>-<sys>-<abi>`, where:
    ///
    /// * `<arch>`   x86, arm, thumb, mips, etc.
    /// * `<sub>`    for example on ARM: v5, v6m, v7a, v7m, etc.
    /// * `<vendor>` pc, apple, nvidia, ibm, etc.
    /// * `<sys>`    none, linux, win32, darwin, cuda, etc.
    /// * `<abi>`    eabi, gnu, android, macho, elf, etc.
    ///
    /// *Defaults* to `$TARGET` environment variable, if set (always is in cargo build scripts).
    ///
    /// Corresponds to the `-mtriple` option of `llc`.
    pub triple: String,
    /// Target CPU to generate machine code for
    ///
    /// *Default* is chosen depending on the target `triple`.
    ///
    /// Corresponds to the `-mcpu` option of `llc`.
    pub cpu: String,
    /// Capabilities of the target code is generated for
    ///
    /// Format of this field is the same as the format for `-mattr` option: +feature enables a
    /// feature, -feature disables it. Each feature is delimited by a comma.
    ///
    /// Sample string: `+sse,+sse2,+sse3,-avx`.
    ///
    /// *Default* is chosen depending on the target `triple`.
    ///
    /// Corresponds to the `-mattr` option of `llc`.
    pub attr: String,
    /// Code generation
    ///
    /// *Defaults* to `CodegenModel::Default`.
    ///
    /// Corresponds to the `-code-model` option of `llc`.
    pub model: CodegenModel,
    /// Relocation model
    ///
    /// *Defaults* to `Relocations::Default`.
    ///
    /// Corresponds to the `-relocation-model` option of `llc`.
    pub reloc: Relocations,
    /// Code optimisation level
    ///
    /// *Defaults* to the same level as specified in the `$OPT_LEVEL` environment variable (set by
    /// cargo) and `Optimisation::O0` if not set.
    ///
    /// Corresponds to the `-O` option of `llc`.
    pub opt: Optimisation,
    /// Name of the archive section to insert generated object into
    pub ar_section_name: String,
}

impl Default for BuildOptions {
    fn default() -> BuildOptions {
        use std::env::var;
        BuildOptions {
            triple: var("TARGET").unwrap_or(String::new()),
            cpu: String::new(),
            attr: String::new(),
            model: CodegenModel::Default,
            reloc: Relocations::Default,
            opt: match var("OPT_LEVEL").ok().and_then(|v| v.parse().ok()).unwrap_or(0u64) {
                0 => Optimisation::O0,
                1 => Optimisation::O1,
                2 => Optimisation::O2,
                3 | _ => Optimisation::O3,
            },
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

/// Produce a static library (archive) containing machine code
///
/// The input files must be well formed LLVM-IR files or LLVM bytecode. Format of the input file
/// is autodetected.
pub fn build_archive<'a, P: 'a, I>(archive: P, iter: I)
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
                                     ArchiveKind::Gnu);
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
