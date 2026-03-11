use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let cvc5_dir = find_cvc5_dir();
    let expected = read_expected_cvc5_version();
    check_cvc5_version(&cvc5_dir, &expected);

    let include_dir = cvc5_dir.join("include");
    let build_dir = cvc5_dir.join("build");
    let build_include_dir = build_dir.join("include");

    // Link against the static library
    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("src").display()
    );
    println!("cargo:rustc-link-lib=static=cvc5");

    // Link parser library
    if cfg!(feature = "parser") {
        println!(
            "cargo:rustc-link-search=native={}",
            build_dir.join("src/parser").display()
        );
        println!("cargo:rustc-link-lib=static=cvc5parser");
    }

    // Link dependencies
    let deps_lib = build_dir.join("deps/lib");
    if deps_lib.exists() {
        println!("cargo:rustc-link-search=native={}", deps_lib.display());
        for lib in &["cadical", "picpoly", "picpolyxx", "gmp"] {
            let path = deps_lib.join(format!("lib{lib}.a"));
            if path.exists() {
                println!("cargo:rustc-link-lib=static={lib}");
            }
        }
    }

    // Link C++ stdlib
    link_cxx_stdlib();

    // Generate bindings from the C API header
    let header = include_dir.join("cvc5/c/cvc5.h");
    assert!(
        header.exists(),
        "cvc5 C header not found at {}",
        header.display()
    );

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("-I{}", build_include_dir.display()))
        .clang_arg("-DCVC5_STATIC_DEFINE")
        .allowlist_function("cvc5_.*")
        .allowlist_type("Cvc5.*")
        .allowlist_var("CVC5_.*")
        .rustified_enum("Cvc5Kind")
        .rustified_enum("Cvc5SortKind")
        .rustified_enum("Cvc5RoundingMode")
        .rustified_enum("Cvc5UnknownExplanation")
        .rustified_enum("Cvc5BlockModelsMode")
        .rustified_enum("Cvc5LearnedLitType")
        .rustified_enum("Cvc5ProofComponent")
        .rustified_enum("Cvc5ProofFormat")
        .rustified_enum("Cvc5ProofRule")
        .rustified_enum("Cvc5SkolemId")
        .rustified_enum("Cvc5FindSynthTarget")
        .rustified_enum("Cvc5InputLanguage")
        .generate_comments(true)
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate cvc5 bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Generate parser bindings if feature enabled
    if cfg!(feature = "parser") {
        let parser_header = include_dir.join("cvc5/c/cvc5_parser.h");
        if parser_header.exists() {
            let parser_bindings = bindgen::Builder::default()
                .header(parser_header.to_string_lossy())
                .clang_arg(format!("-I{}", include_dir.display()))
                .clang_arg(format!("-I{}", build_include_dir.display()))
                .clang_arg("-DCVC5_STATIC_DEFINE")
                .allowlist_function("cvc5_parser_.*")
                .allowlist_function("cvc5_cmd_.*")
                .allowlist_function("cvc5_symbol_manager_.*")
                .allowlist_function("cvc5_sm_.*")
                .allowlist_type("Cvc5InputParser")
                .allowlist_type("Cvc5SymbolManager")
                .allowlist_type("Cvc5Command")
                .allowlist_type("cvc5_cmd_t")
                .blocklist_type("Cvc5")
                .blocklist_type("Cvc5TermManager")
                .blocklist_type("Cvc5Sort")
                .blocklist_type("cvc5_sort_t")
                .blocklist_type("Cvc5Term")
                .blocklist_type("cvc5_term_t")
                .blocklist_type("Cvc5InputLanguage")
                .raw_line("use super::*;")
                .generate_comments(true)
                .derive_default(true)
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .generate()
                .expect("Unable to generate cvc5 parser bindings");

            parser_bindings
                .write_to_file(out_path.join("parser_bindings.rs"))
                .expect("Couldn't write parser bindings!");
        }
    }
}

fn read_expected_cvc5_version() -> String {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest).expect("Failed to read Cargo.toml");
    let table: toml::Table = content.parse().expect("Failed to parse Cargo.toml");
    table["package"]["metadata"]["cvc5"]["version"]
        .as_str()
        .expect("Missing version in [package.metadata.cvc5]")
        .to_string()
}

fn check_cvc5_version(cvc5_dir: &PathBuf, expected: &str) {
    let version_file = cvc5_dir.join("cmake/version-base.cmake");
    assert!(
        version_file.exists(),
        "cvc5 version file not found at {}",
        version_file.display()
    );

    let content = std::fs::read_to_string(&version_file)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", version_file.display()));

    let version = content
        .lines()
        .find_map(|line| {
            line.strip_prefix("set(CVC5_LAST_RELEASE \"")
                .and_then(|rest| rest.strip_suffix("\")"))
        })
        .unwrap_or_else(|| panic!("CVC5_LAST_RELEASE not found in {}", version_file.display()));

    assert_eq!(
        version, expected,
        "cvc5 version mismatch: found {version}, expected {expected}. \
         Update the cvc5 submodule or change version in [package.metadata.cvc5] in Cargo.toml."
    );

    println!("cargo:rerun-if-changed={}", version_file.display());
}

fn find_cvc5_dir() -> PathBuf {
    // 1. Check CVC5_DIR env var
    println!("cargo:rerun-if-env-changed=CVC5_DIR");
    if let Ok(dir) = env::var("CVC5_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }

    // 2. Check for sibling cvc5 directory (submodule)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let sibling = manifest_dir.parent().unwrap().join("cvc5");
    if sibling.join("include/cvc5/c/cvc5.h").exists() {
        return sibling;
    }

    panic!(
        "Could not find cvc5. Set CVC5_DIR to the cvc5 source root \
         (containing include/ and build/ directories), or place a built \
         cvc5 checkout as a sibling directory."
    );
}

fn link_cxx_stdlib() {
    let cxx = match env::var("CXXSTDLIB") {
        Ok(s) if s.is_empty() => None,
        Ok(s) => Some(s),
        Err(_) => {
            let target = env::var("TARGET").unwrap();
            if target.contains("msvc") {
                None
            } else if target.contains("apple") || target.contains("freebsd") {
                Some("c++".to_string())
            } else {
                Some("stdc++".to_string())
            }
        }
    };
    if let Some(cxx) = cxx {
        println!("cargo:rustc-link-lib={cxx}");
    }
}
