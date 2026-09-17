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

use crate::error::{Error, Result, has_error, last_error};
use std::ffi::CStr;
use std::os::raw::c_char;

/// Report an unexpected `NULL` from the C API and panic.
///
/// cvc5 records why a call failed in thread-local state and clears it on entry
/// to the *next* call, so the message is only readable here because no cvc5 call
/// happens between the failing one and this guard.
#[cold]
#[inline(never)]
fn unexpected_null(what: &str) -> ! {
    match last_error() {
        Some(msg) => panic!("cvc5 call failed while producing `{what}`: {msg}"),
        None => panic!(
            "cvc5 C API returned NULL for `{what}` but reported no error. This is \
             an unexpected result rather than a failed call."
        ),
    }
}

/// Gate a value on cvc5's thread-local error state.
///
/// Call immediately after the C function, before constructing any wrapper: the
/// state reflects only the most recent cvc5 call, and `from_raw` would panic on
/// a `NULL` that this turns into an `Err`.
///
/// For values that are about to be wrapped in an owning type, prefer [`wrap`],
/// which pairs this check with the constructor.
#[inline]
pub(crate) fn checked<T>(value: T, what: &str) -> Result<T> {
    if has_error() {
        Err(Error::from_state(what))
    } else {
        Ok(value)
    }
}

/// A wrapper type built from a raw cvc5 pointer.
///
/// Implemented by delegating to each type's infallible `from_raw`, which stays
/// infallible because `Clone` and the many non-fallible getters both need it.
pub(crate) trait FromRaw: Sized {
    type Raw;
    fn wrap_raw(raw: Self::Raw) -> Self;
}

/// Gate a raw pointer on the error state and wrap it, in one step.
///
/// This is the shape almost every fallible getter and constructor needs:
/// `wrap(unsafe { cvc5_... }, "name")` instead of hand-composing [`checked`]
/// with `from_raw`.
#[inline]
pub(crate) fn wrap<T: FromRaw>(raw: T::Raw, what: &str) -> Result<T> {
    checked(raw, what).map(T::wrap_raw)
}

macro_rules! impl_from_raw {
    ($($ty:ty : $raw:ty),* $(,)?) => {
        $(impl FromRaw for $ty {
            type Raw = $raw;
            #[inline]
            fn wrap_raw(raw: Self::Raw) -> Self {
                Self::from_raw(raw)
            }
        })*
    };
}

impl_from_raw! {
    crate::Term<'_>: cvc5_sys::Term,
    crate::Sort<'_>: cvc5_sys::Sort,
    crate::Op<'_>: cvc5_sys::Op,
    crate::Datatype<'_>: cvc5_sys::Datatype,
    crate::DatatypeConstructor<'_>: cvc5_sys::DatatypeConstructor,
    crate::DatatypeSelector<'_>: cvc5_sys::DatatypeSelector,
    crate::DatatypeDecl<'_>: cvc5_sys::DatatypeDecl,
    crate::DatatypeConstructorDecl<'_>: cvc5_sys::DatatypeConstructorDecl,
    crate::Proof<'_>: cvc5_sys::Proof,
    crate::Grammar<'_>: cvc5_sys::Grammar,
    crate::SatResult<'_>: cvc5_sys::Result,
    crate::SynthResult<'_>: cvc5_sys::SynthResult,
    crate::Statistics: cvc5_sys::Statistics,
    crate::Stat: cvc5_sys::Stat,
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
