Ever wanted to build “native” assembly stuff in your cargo build scripts… something gcc crate
cannot quite handle yet?  Welcome to llvm_build_utils which provides a convenient API to pack your
.ll or .bc files into a ready to use archive full of machine code! It doesn’t even need LLVM
installation and works on stable Rust¹!

¹: May break between versions or be incompatible with some versions of Rust, though. We’ll try to
document such breakages in the table below.

# Compatibility table

| Rustc version | This Library  |
| ------------- | ------------- |
| 1.8-1.11      | 0.1.0         |

# Using llvm_build_utils

First, you'll want to both add a build script for your crate (build.rs) and also add this crate to
your Cargo.toml via:

```toml
[package]
# ...
build = "build.rs"

[build-dependencies]
llvm_build_utils = "0.1"
```

Then write your `build.rs` like this:

```rust
extern crate llvm_build_utils;
use llvm_build_utils::*;

fn main() {
    build_archive("libyourthing.a", &[
    ("input.ll", BuildOptions {
        // customise how the file is built
        ..BuildOptions::default()
    })/*, ("input2.ll", ...
        // more .ll files to be built into the archive in same format as first one
    */]).expect("error happened");
}
```

Running a `cargo build` should produce `libyourthing.a` which then may be linked to your Rust
executable/library.

# License

llvm_build_utils is distributed under ISC (MIT-like) or Apache (version 2.0) license at your
choice.
