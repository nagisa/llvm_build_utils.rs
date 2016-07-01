extern crate llvm_build_utils;

use llvm_build_utils::*;

#[test]
fn test_build() {
    build_archive("libtest.a", &[("tests/test.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        ..BuildOptions::default()
    }), ("tests/test.ll", BuildOptions {
        triple: String::from("i386-unknown-linux-gnu"),
        ..BuildOptions::default()
    })]).unwrap();
}

#[test]
fn test_cpu_attr() {
    build_archive("librand.a", &[("tests/rdrand.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdrnd"),
        ..BuildOptions::default()
    }), ("tests/rdseed.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdseed"),
        ..BuildOptions::default()
    })]).unwrap();
}

#[test]
fn allow_dynamic_dispatch() {
    use std::path::*;
    let pb = PathBuf::from("libtest.a");
    let t1 = Path::new("tests/test.ll");
    build_archive(&pb as &AsRef<Path>,
    &[(&t1 as &AsRef<Path>, BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        ..BuildOptions::default()
    }), (&"tests/test.ll" as &AsRef<Path>, BuildOptions {
        triple: String::from("i386-unknown-linux-gnu"),
        ..BuildOptions::default()
    })]).unwrap();
}

#[test]
fn test_optimisation() {
    build_archive("librandopt.a", &[("tests/rdrand.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdrnd"),
        opt: CodeGenOptLevel::O3,
        ..BuildOptions::default()
    })]).unwrap();
}

#[test]
fn test_wrong_things_fail_1() {
    build_archive("fail.a", &[("tests/does_not_exist_for_sure.ll",
                     BuildOptions::default())]).err().unwrap();
}

#[test]
fn test_wrong_things_fail_2() {
    build_archive("/", &[("tests/test.ll",
                     BuildOptions::default())]).err().unwrap();
}

#[test]
fn test_wrong_things_fail_3() {
    build_archive("banana.a/", &[("tests/test.ll",
                     BuildOptions::default())]).err().unwrap();
}

#[test]
fn test_wrong_things_fail_4() {
    build_archive("test.a", &[("tests/test.ll", BuildOptions {
        triple: String::from("some weird triple this is"),
        ..BuildOptions::default()
    })]).err().unwrap();
}
