/* EVMC: Ethereum Client-VM Connector API.
 * Copyright 2019 The EVMC Authors.
 * Licensed under the Apache License, Version 2.0.
 */

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn gen_bindings() {
    let v = vec!["--sysroot=/tmp/wasi-sysroot", "-Wl,--allow-undefined-file=/tmp/wasi-sysroot/share/wasm32-wasi/undefined-symbols.txt"];
    let bindings = bindgen::Builder::default()
        .header("evmc.h")
        // See https://github.com/rust-lang-nursery/rust-bindgen/issues/947
        .trust_clang_mangling(false)
        .generate_comments(true)
        // https://github.com/rust-lang-nursery/rust-bindgen/issues/947#issuecomment-327100002
        .layout_tests(false)
        // do not generate an empty enum for EVMC_ABI_VERSION
        .constified_enum("")
        // generate Rust enums for each evmc enum
        .rustified_enum("*")
        // force deriving the Hash trait on basic types (address, bytes32)
        .derive_hash(true)
        // force deriving the PratialEq trait on basic types (address, bytes32)
        .derive_partialeq(true)
        .blacklist_type("evmc_host_context")
        .whitelist_type("evmc_.*")
        .whitelist_function("evmc_.*")
        .whitelist_var("EVMC_ABI_VERSION")
        .clang_args(v.into_iter())
        // TODO: consider removing this
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    gen_bindings();
}
