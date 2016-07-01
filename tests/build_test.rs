extern crate llvm_build_utils;

use llvm_build_utils::*;

#[test]
fn test_build() {
    build_archive(&[("tests/test.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        ..BuildOptions::default()
    }), ("tests/test.ll", BuildOptions {
        triple: String::from("i386-unknown-linux-gnu"),
        ..BuildOptions::default()
    })], "test.a").unwrap();
}

#[test]
fn test_cpu_attr() {
    build_archive(&[("tests/rdrand.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdrnd"),
        ..BuildOptions::default()
    }), ("tests/rdseed.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdseed"),
        ..BuildOptions::default()
    })], "rand.a").unwrap();
}

#[test]
fn test_optimisation() {
    build_archive(&[("tests/rdrand.ll", BuildOptions {
        triple: String::from("x86_64-unknown-linux-gnu"),
        cpu: String::from("x86-64"),
        attr: String::from("+rdrnd"),
        opt: CodeGenOptLevel::O3,
        ..BuildOptions::default()
    })], "rand-opt.a").unwrap();
}

#[test]
fn test_wrong_things_fail_1() {
    build_archive(&[("tests/does_not_exist_for_sure.ll",
                     BuildOptions::default())], "fail.a").err().unwrap();
}

#[test]
fn test_wrong_things_fail_2() {
    build_archive(&[("tests/test.ll",
                     BuildOptions::default())], "/").err().unwrap();
}

#[test]
fn test_wrong_things_fail_3() {
    build_archive(&[("tests/test.ll",
                     BuildOptions::default())], "./banana.a/").err().unwrap();
}

#[test]
fn test_wrong_things_fail_4() {
    build_archive(&[("tests/test.ll", BuildOptions {
        triple: String::from("some weird triple this is"),
        ..BuildOptions::default()
    })], "test.a").err().unwrap();
}
