//! Internal guards for raw pointers returned by the cvc5 C API.
//!
//! # Why these exist
//!
//! The cvc5 C API signals "no value" and (depending on the cvc5 version)
//! "an error occurred" through the same channel: a `NULL` return.
//!
//! * Up to and including cvc5 1.3.4, an API error printed to `stderr` and
//!   called `exit(EXIT_FAILURE)` (`Cvc5CApiAbortStream` in
//!   `src/api/c/cvc5_checks.h`), so `NULL` only ever meant "legitimately no
//!   value" — for example [`crate::Solver::get_interpolant`] when no
//!   interpolant exists.
//! * cvc5 `main` (PR #12724, merged after the 1.3.4 release) removed that
//!   abort. `CVC5_CAPI_TRY_CATCH_END` now records the message in thread-local
//!   state and lets the function fall through to its default-initialized
//!   return value. That makes `NULL` reachable from *every* pointer-returning
//!   C function, and the caller is expected to poll `cvc5_has_error()`.
//!
//! These helpers are written so the same binding is correct under both
//! contracts. Where `NULL` is a documented outcome the caller checks for it
//! explicitly and yields `None`/empty; everywhere else an unexpected `NULL` is
//! turned into a panic rather than being wrapped and dereferenced later.
//! Panicking is a strict improvement over both upstream behaviours: it is
//! catchable, it names the operation that failed, and it does not take the
//! process down or defer the failure to an unrelated line.

use std::ffi::CStr;
use std::os::raw::c_char;

/// Report an unexpected `NULL` from the C API and panic.
#[cold]
#[inline(never)]
fn unexpected_null(what: &str) -> ! {
    panic!(
        "cvc5 C API returned NULL for `{what}`. This means the underlying call \
         failed. On cvc5 builds that capture errors instead of aborting \
         (post-1.3.4), the reason is available from `cvc5_get_error_message()`."
    )
}

/// Guard an object pointer that the C API is expected to always populate.
///
/// Panics if `ptr` is `NULL`. Use this in every `from_raw` constructor so a
/// failed C call surfaces here instead of as a use-after-free or segfault at
/// the next method call or on `Drop`.
#[inline]
pub(crate) fn non_null<T>(ptr: *mut T, what: &str) -> *mut T {
    if ptr.is_null() {
        unexpected_null(what);
    }
    ptr
}

/// View a C-owned array as a slice, treating `NULL` as empty.
///
/// A zero length is also treated as empty without inspecting `ptr`: several
/// cvc5 getters return `res.data()` on a `static thread_local std::vector`,
/// which is `NULL` for a vector that has never allocated but non-`NULL` after
/// any earlier non-empty result. Callers must not rely on either.
///
/// # Safety
///
/// If `ptr` is non-`NULL` and `len > 0`, `ptr` must point to `len` initialized
/// values of `T` that stay valid for `'a`. cvc5 keeps these buffers in
/// thread-local storage that the *next* call on the same thread overwrites, so
/// `'a` must not outlive the surrounding statement.
#[inline]
pub(crate) unsafe fn raw_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Copy a C string into an owned [`String`], panicking on `NULL`.
///
/// # Safety
///
/// If non-`NULL`, `ptr` must point to a NUL-terminated string valid for the
/// duration of the call.
#[inline]
pub(crate) unsafe fn cstr_to_string(ptr: *const c_char, what: &str) -> String {
    if ptr.is_null() {
        unexpected_null(what);
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// Borrow a C string as a `&str`, mapping `NULL` (and invalid UTF-8) to `""`.
///
/// For accessors whose value is genuinely optional — e.g. the symbol of a sort
/// or term that has none, where the paired `has_symbol()` predicate is the
/// intended way to discriminate. Returning `""` keeps those signatures
/// infallible while removing the `NULL` dereference.
///
/// # Safety
///
/// If non-`NULL`, `ptr` must point to a NUL-terminated string that stays valid
/// for `'a`. cvc5 returns these from thread-local buffers that the next call on
/// the same thread overwrites.
#[inline]
pub(crate) unsafe fn cstr_or_empty<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("")
}
