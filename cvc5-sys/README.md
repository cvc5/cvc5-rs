# cvc5-sys

Low-level FFI bindings for the [cvc5](https://cvc5.github.io/) SMT solver, generated
via [bindgen](https://github.com/rust-lang/rust-bindgen) from the cvc5 C API header (`cvc5/c/cvc5.h`).

For a safe, idiomatic Rust API, see the higher-level [`cvc5-rs`](https://github.com/cvc5/cvc5-rs) crate.

## Prerequisites

This crate wraps cvc5 1.3.1 (the expected version is declared in `Cargo.toml` under
`[package.metadata.cvc5]`). If cvc5 has not been compiled yet, the build script runs
`configure.sh --static --auto-download` and `make` automatically. You need:

- A C/C++ compiler (GCC or Clang)
- CMake ≥ 3.16
- libclang (required by bindgen)
- Git (for automatic source download when installed from crates.io)

## Source Acquisition

The build script locates the cvc5 source in the following order:

1. **`CVC5_DIR` environment variable** — set to the cvc5 source root containing `include/` and `build/` directories.
2. **`cvc5/` subdirectory** inside the `cvc5-sys` crate — the default when using the git submodule.
3. **Sibling `cvc5/` directory** — a built cvc5 checkout next to the `cvc5-sys` crate (workspace layout).
4. **Automatic clone** — if none of the above are found, the build script clones the matching cvc5 release tag from GitHub.

```bash
# Option 1: explicit path
CVC5_DIR=/path/to/cvc5 cargo build

# Option 2: submodule (no env var needed)
git clone --recurse-submodules https://github.com/cvc5/cvc5-rs.git
cd cvc5-rs && cargo build

# Option 3: from crates.io (auto-clones cvc5)
cargo add cvc5-sys
cargo build
```

An application can set `CVC5_DIR` in its `.cargo/config.toml` to point to a local cvc5 checkout:

```toml
[env]
CVC5_DIR = { value = "cvc5", relative = true }
```

## Features

| Feature  | Description                                       |
|----------|---------------------------------------------------|
| `parser` | Also generate and link bindings for `cvc5parser`. |

Enable with:

```bash
cargo build --features parser
```

## Usage

All symbols mirror the C API directly. Every call is `unsafe`.

```rust
use cvc5_sys::*;

unsafe {
    let tm = cvc5_term_manager_new();
    let slv = cvc5_new(tm);

    cvc5_set_logic(slv, c"QF_LIA".as_ptr());
    cvc5_set_option(slv, c"produce-models".as_ptr(), c"true".as_ptr());

    let int_sort = cvc5_get_integer_sort(tm);
    let x = cvc5_mk_const(tm, int_sort, c"x".as_ptr());
    let zero = cvc5_mk_integer_int64(tm, 0);

    let gt = cvc5_mk_term(tm, Cvc5Kind::CVC5_KIND_GT, 2, [x, zero].as_ptr());
    cvc5_assert_formula(slv, gt);

    let result = cvc5_check_sat(slv);
    assert!(cvc5_result_is_sat(result));

    cvc5_delete(slv);
    cvc5_term_manager_delete(tm);
}
```

## Linked Libraries

The build script statically links:

- `libcvc5` (and `libcvc5parser` with the `parser` feature)
- Bundled dependencies when present: `cadical`, `picpoly`, `picpolyxx`, `gmp`
- The platform C++ standard library (`libc++` on macOS, `libstdc++` on Linux)

## License

BSD-3-Clause — see [LICENSE](../LICENSE).
