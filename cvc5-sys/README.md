# cvc5-sys

Low-level FFI bindings for the [cvc5](https://cvc5.github.io/) SMT solver, generated
via [bindgen](https://github.com/rust-lang/rust-bindgen) from the cvc5 C API header (`cvc5/c/cvc5.h`).

For a safe, idiomatic Rust API, see the higher-level [`cvc5-rs`](https://github.com/cvc5/cvc5-rs) crate.

## Prerequisites

When the `static` feature is enabled, this crate wraps cvc5 1.3.1 (the expected
version is declared in `Cargo.toml` under `[package.metadata.cvc5]`). If cvc5
has not been compiled yet, the build script builds cvc5 automatically. You need:

- A C/C++ compiler (GCC or Clang)
- CMake ≥ 3.16
- libclang (required by bindgen)
- Git (for automatic source download when installed from crates.io)

Without the `static` feature, the build script does not build cvc5 from source.
Instead it expects cvc5 to be installed on the system or in a path pointed to by `CVC5_LIB_DIR`,
and discovers headers via `CVC5_INCLUDE_DIR` or by asking the C compiler.

## Source acquisition (static feature)

The build script locates the cvc5 source in the following order:

1. **`CVC5_DIR` environment variable** — set to the cvc5 source root.
2. **`cvc5/` subdirectory** inside the `cvc5-sys` crate — the default when using the git submodule.
3. **Automatic clone** — if none of the above are found, the build script clones the matching cvc5 release tag from
   GitHub into `OUT_DIR`.

```bash
# Option 1: explicit path
CVC5_DIR=/path/to/cvc5 cargo build --features static

# Option 2: submodule (no env var needed)
git clone --recurse-submodules https://github.com/cvc5/cvc5-rs.git
cd cvc5-rs && cargo build --features static

# Option 3: from crates.io (auto-clones cvc5)
cargo add cvc5-sys --features static
cargo build
```

An application can set `CVC5_DIR` in its `.cargo/config.toml` to point to a local cvc5 checkout:

```toml
[env]
CVC5_DIR = { value = "cvc5", relative = true }
```

## Linking against a prebuilt cvc5

If you already have cvc5 built and installed, you can skip the automatic build by setting
`CVC5_LIB_DIR` to the directory containing the libraries (`libcvc5.a` for static, or
`libcvc5.so`/`libcvc5.dylib` for dynamic):

```bash
CVC5_LIB_DIR=/path/to/cvc5/build/install/lib cargo build --features static # for static linking
```

or

```bash
CVC5_LIB_DIR=/path/to/cvc5/build/install/lib cargo build # for dynamic linking
```

Headers are resolved in this order:

1. `CVC5_INCLUDE_DIR` environment variable (if set)
2. `$CVC5_LIB_DIR/../include` (the conventional install layout)
3. Compiler discovery — the build script asks the C compiler to locate `cvc5/c/cvc5.h` on the system include path

```bash
# Override both paths explicitly
CVC5_LIB_DIR=/path/to/libs CVC5_INCLUDE_DIR=/path/to/include cargo build
```

## Features

| Feature  | Description                                                                                                                                                                                 |
|----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `static` | Statically link cvc5. Enables automatic source build if cvc5 is not already compiled.<br/> Otherwise, the crate employs dynamic linking to cvc5 and assume some local installation of cvc5. |
| `parser` | Also generate and link bindings for `cvc5parser`.                                                                                                                                           |

Enable with:

```bash
cargo build --features static,parser
```

## Usage

All symbols mirror the C API directly. Every call is `unsafe`.

```rust
use cvc5_sys::*;

unsafe {
    let tm = term_manager_new();
    let slv = new(tm);

    set_logic(slv, c"QF_LIA".as_ptr());
    set_option(slv, c"produce-models".as_ptr(), c"true".as_ptr());

    let int_sort = get_integer_sort(tm);
    let x = mk_const(tm, int_sort, c"x".as_ptr());
    let zero = mk_integer_int64(tm, 0);

    let gt = mk_term(tm, Kind::Gt, 2, [x, zero].as_ptr());
    assert_formula(slv, gt);

    let result = check_sat(slv);
    assert!(result_is_sat(result));

    delete(slv);
    term_manager_delete(tm);
}
```

## Linked libraries

When the `static` feature is enabled, the build script statically links:

- `libcvc5` (and `libcvc5parser` with the `parser` feature)
- Bundled dependencies: `cadical`, `picpoly`, `picpolyxx`, `gmp`
- The platform C++ standard library (`libc++` on macOS, `libstdc++` on Linux)
- `libstdc++_nonshared` on RHEL/CentOS with GCC toolsets (detected automatically)

The build script supports both `lib/` and `lib64/` install layouts.

## License

BSD-3-Clause — see [LICENSE](../LICENSE).
