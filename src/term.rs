use cvc5_sys::*;
use std::fmt;

use crate::Sort;

/// A cvc5 term (expression).
#[derive(Clone, Copy)]
pub struct Term {
    pub(crate) inner: Cvc5Term,
}

impl Term {
    pub(crate) fn from_raw(raw: Cvc5Term) -> Self {
        Self { inner: raw }
    }

    /// Get the kind of this term.
    pub fn kind(&self) -> Cvc5Kind {
        unsafe { cvc5_term_get_kind(self.inner) }
    }

    /// Get the sort of this term.
    pub fn sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_term_get_sort(self.inner) })
    }

    /// Get the id of this term.
    pub fn id(&self) -> u64 {
        unsafe { cvc5_term_get_id(self.inner) }
    }

    /// Get the number of children.
    pub fn num_children(&self) -> usize {
        unsafe { cvc5_term_get_num_children(self.inner) }
    }

    /// Get child at index.
    pub fn child(&self, index: usize) -> Term {
        Term::from_raw(unsafe { cvc5_term_get_child(self.inner, index) })
    }

    /// Check if this term has a symbol.
    pub fn has_symbol(&self) -> bool {
        unsafe { cvc5_term_has_symbol(self.inner) }
    }

    /// Get the symbol of this term.
    pub fn symbol(&self) -> &str {
        unsafe {
            let s = cvc5_term_get_symbol(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    /// Check if this is a Boolean value.
    pub fn is_boolean_value(&self) -> bool {
        unsafe { cvc5_term_is_boolean_value(self.inner) }
    }

    /// Get the Boolean value.
    pub fn boolean_value(&self) -> bool {
        unsafe { cvc5_term_get_boolean_value(self.inner) }
    }

    /// Check if this is an i32 integer value.
    pub fn is_int32_value(&self) -> bool {
        unsafe { cvc5_term_is_int32_value(self.inner) }
    }

    /// Get the i32 value.
    pub fn int32_value(&self) -> i32 {
        unsafe { cvc5_term_get_int32_value(self.inner) }
    }

    /// Check if this is an i64 integer value.
    pub fn is_int64_value(&self) -> bool {
        unsafe { cvc5_term_is_int64_value(self.inner) }
    }

    /// Get the i64 value.
    pub fn int64_value(&self) -> i64 {
        unsafe { cvc5_term_get_int64_value(self.inner) }
    }

    /// Check if this is an integer value.
    pub fn is_integer_value(&self) -> bool {
        unsafe { cvc5_term_is_integer_value(self.inner) }
    }

    /// Get the integer value as a string.
    pub fn integer_value(&self) -> String {
        unsafe {
            let s = cvc5_term_get_integer_value(self.inner);
            std::ffi::CStr::from_ptr(s)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Check if this is a real value.
    pub fn is_real_value(&self) -> bool {
        unsafe { cvc5_term_is_real_value(self.inner) }
    }

    /// Get the real value as a string.
    pub fn real_value(&self) -> String {
        unsafe {
            let s = cvc5_term_get_real_value(self.inner);
            std::ffi::CStr::from_ptr(s)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Check if this is a string value.
    pub fn is_string_value(&self) -> bool {
        unsafe { cvc5_term_is_string_value(self.inner) }
    }

    /// Check if this is a bit-vector value.
    pub fn is_bv_value(&self) -> bool {
        unsafe { cvc5_term_is_bv_value(self.inner) }
    }

    /// Get the bit-vector value as a string in the given base (2, 10, or 16).
    pub fn bv_value(&self, base: u32) -> String {
        unsafe {
            let s = cvc5_term_get_bv_value(self.inner, base);
            std::ffi::CStr::from_ptr(s)
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_term_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Term({self})")
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_term_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Term {}

impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_term_hash(self.inner) }.hash(state);
    }
}
