use cvc5_sys::{Cvc5Sort, Cvc5SortKind, cvc5_sort_get_kind, cvc5_sort_to_string};
use std::fmt;

/// A cvc5 sort (type).
#[derive(Clone, Copy)]
pub struct Sort {
    pub(crate) inner: Cvc5Sort,
}

impl Sort {
    pub(crate) fn from_raw(raw: Cvc5Sort) -> Self {
        Self { inner: raw }
    }

    /// Get the kind of this sort.
    pub fn kind(&self) -> Cvc5SortKind {
        unsafe { cvc5_sort_get_kind(self.inner) }
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_sort_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sort({self})")
    }
}
