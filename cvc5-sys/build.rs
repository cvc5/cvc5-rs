use std::path::Path;
use std::{env, path::PathBuf, process::Command};

use bindgen::callbacks::ParseCallbacks;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CVC5_LIB_DIR");
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    // On docs.rs, skip building/linking cvc5 and use pre-generated bindings.
    if env::var("DOCS_RS").is_ok() {
        let out = PathBuf::from(env::var("OUT_DIR").unwrap());
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        std::fs::copy(
            manifest.join("prebuilt/bindings.rs"),
            out.join("bindings.rs"),
        )
        .expect("Failed to copy pre-generated bindings");
        if cfg!(feature = "parser") {
            std::fs::copy(
                manifest.join("prebuilt/parser_bindings.rs"),
                out.join("parser_bindings.rs"),
            )
            .expect("Failed to copy pre-generated parser bindings");
        }
        return;
    }

    // If CVC5_LIB_DIR is set, link directly without building cvc5.
    if let Ok(lib_dir) = env::var("CVC5_LIB_DIR") {
        link_prebuilt(&PathBuf::from(lib_dir));
        return;
    }

    let cvc5_dir = find_cvc5_dir();
    let expected = read_expected_cvc5_version();
    check_cvc5_version(&cvc5_dir, &expected);
    ensure_cvc5_built(&cvc5_dir);

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

    // Generate bindings
    generate_bindings(&include_dir, &build_include_dir);
}

/// Renames enums
#[derive(Debug)]
struct RenamingCallback;
impl ParseCallbacks for RenamingCallback {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let enum_name = enum_name?;
        // For some reason, some enums start with `enum `
        let name = enum_name.strip_prefix("enum ").unwrap_or(enum_name);
        match name {
            "Cvc5Kind" => Some(
                original_variant_name
                    .strip_prefix("CVC5_KIND_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5SortKind" => Some(
                original_variant_name
                    .strip_prefix("CVC5_SORT_KIND_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5RoundingMode" => Some(
                original_variant_name
                    .strip_prefix("CVC5_RM_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5UnknownExplanation" => Some(
                original_variant_name
                    .strip_prefix("CVC5_UNKNOWN_EXPLANATION_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5BlockModelsMode" => Some(
                original_variant_name
                    .strip_prefix("CVC5_BLOCK_MODELS_MODE_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5LearnedLitType" => Some(
                original_variant_name
                    .strip_prefix("CVC5_LEARNED_LIT_TYPE_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5ProofComponent" => Some(
                original_variant_name
                    .strip_prefix("CVC5_PROOF_COMPONENT_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5ProofFormat" => Some(
                original_variant_name
                    .strip_prefix("CVC5_PROOF_FORMAT_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5ProofRule" => Some(
                original_variant_name
                    .strip_prefix("CVC5_PROOF_RULE_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5ProofRewriteRule" => Some(
                original_variant_name
                    .strip_prefix("CVC5_PROOF_REWRITE_RULE_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5SkolemId" => Some(
                original_variant_name
                    .strip_prefix("CVC5_SKOLEM_ID_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5FindSynthTarget" => Some(
                original_variant_name
                    .strip_prefix("CVC5_FIND_SYNTH_TARGET_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5InputLanguage" => Some(
                original_variant_name
                    .strip_prefix("CVC5_INPUT_LANGUAGE_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5OptionCategory" => Some(
                original_variant_name
                    .strip_prefix("CVC5_OPTION_CATEGORY_")
                    .expect("Prefix")
                    .to_string(),
            ),
            "Cvc5OptionInfoKind" => Some(
                original_variant_name
                    .strip_prefix("CVC5_OPTION_INFO_")
                    .expect("Prefix")
                    .to_string(),
            ),
            _ => None,
        }
    }
}

fn generate_bindings(include_dir: &Path, build_include_dir: &Path) {
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
        .parse_callbacks(Box::new(RenamingCallback))
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
        .rustified_enum("Cvc5ProofRewriteRule")
        .rustified_enum("Cvc5SkolemId")
        .rustified_enum("Cvc5FindSynthTarget")
        .rustified_enum("Cvc5InputLanguage")
        .rustified_enum("Cvc5OptionCategory")
        .rustified_enum("Cvc5OptionInfoKind")
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

fn check_cvc5_version(cvc5_dir: &Path, expected: &str) {
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

#[cfg(unix)]
fn ensure_cvc5_built(cvc5_dir: &PathBuf) {
    if cvc5_dir.join("build/src/libcvc5.a").exists() {
        return;
    }

    eprintln!("cvc5 not yet built — running configure and make (this may take a while)...");

    let configure = cvc5_dir.join("configure.sh");
    assert!(
        configure.exists(),
        "configure.sh not found at {}",
        configure.display()
    );

    let build_dir = cvc5_dir.join("build");

    let status = Command::new("bash")
        .arg(&configure)
        .arg("--static")
        .arg("--auto-download")
        .arg(format!("--prefix={}/install", build_dir.display()))
        .arg("-DBUILD_GMP=1")
        .current_dir(cvc5_dir)
        .status()
        .expect("Failed to run configure.sh");
    assert!(status.success(), "cvc5 configure.sh failed");

    let jobs = std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "4".to_string());

    let status = Command::new("make")
        .arg(format!("-j{jobs}"))
        .current_dir(cvc5_dir.join("build"))
        .status()
        .expect("Failed to run make");
    assert!(status.success(), "cvc5 build failed");
}

#[cfg(not(unix))]
fn ensure_cvc5_built(cvc5_dir: &PathBuf) {
    assert!(
        cvc5_dir.join("build/src/libcvc5.a").exists(),
        "cvc5 is not built and automatic building is only supported on Unix. \
         Please build cvc5 manually before running cargo build."
    );
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

    // 2. Check for cvc5 directory inside cvc5-sys (submodule)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let local = manifest_dir.join("cvc5");
    if local.join("include/cvc5/c/cvc5.h").exists() {
        return local;
    }

    // 3. Clone into OUT_DIR (safe during cargo publish)
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("cvc5");
    if out.join("include/cvc5/c/cvc5.h").exists() {
        return out;
    }

    let expected = read_expected_cvc5_version();
    let tag = format!("cvc5-{expected}");
    eprintln!("cvc5 source not found — cloning tag {tag} from GitHub...");

    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", &tag])
        .arg("https://github.com/cvc5/cvc5.git")
        .arg(&out)
        .status()
        .expect("Failed to run git clone — is git installed?");
    assert!(status.success(), "git clone of cvc5 tag {tag} failed");

    out
}

/// Link against prebuilt cvc5 libraries and generate bindings.
///
/// `lib_dir` is the directory containing the static libraries (libcvc5.a, etc.).
/// Headers are located via `CVC5_INCLUDE_DIR` env var, or `<lib_dir>/../include`.
fn link_prebuilt(lib_dir: &Path) {
    println!("cargo:rerun-if-env-changed=CVC5_INCLUDE_DIR");

    assert!(
        lib_dir.exists(),
        "CVC5_LIB_DIR does not exist: {}",
        lib_dir.display()
    );

    // Find include directory
    let include_dir = match env::var("CVC5_INCLUDE_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => lib_dir.join("../include"),
    };
    let include_dir = include_dir.canonicalize().unwrap_or_else(|_| {
        panic!(
            "Include directory not found. Set CVC5_INCLUDE_DIR or ensure \
             {}/include exists.",
            lib_dir.join("..").display()
        )
    });

    // Link
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=cvc5");

    if cfg!(feature = "parser") {
        println!("cargo:rustc-link-lib=static=cvc5parser");
    }

    // Link bundled dependencies if present
    for lib in &["cadical", "picpoly", "picpolyxx", "gmp"] {
        if lib_dir.join(format!("lib{lib}.a")).exists() {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }

    link_cxx_stdlib();
    generate_bindings(&include_dir, &include_dir);
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
