use cvc5_sys::*;
use std::fmt;

use crate::Term;

/// A cvc5 operator (indexed operator).
pub struct Op {
    pub(crate) inner: Cvc5Op,
}

impl Clone for Op {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_op_copy(self.inner) },
        }
    }
}

impl Drop for Op {
    fn drop(&mut self) {
        unsafe { cvc5_op_release(self.inner) }
    }
}

impl Op {
    pub(crate) fn from_raw(raw: Cvc5Op) -> Self {
        Self { inner: raw }
    }

    pub fn kind(&self) -> Cvc5Kind {
        unsafe { cvc5_op_get_kind(self.inner) }
    }

    pub fn copy(&self) -> Op {
        Op::from_raw(unsafe { cvc5_op_copy(self.inner) })
    }
    pub fn is_disequal(&self, other: &Op) -> bool {
        unsafe { cvc5_op_is_disequal(self.inner, other.inner) }
    }

    pub fn is_indexed(&self) -> bool {
        unsafe { cvc5_op_is_indexed(self.inner) }
    }
    pub fn num_indices(&self) -> usize {
        unsafe { cvc5_op_get_num_indices(self.inner) }
    }

    pub fn index(&self, i: usize) -> Term {
        Term::from_raw(unsafe { cvc5_op_get_index(self.inner, i) })
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_op_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Op({self})")
    }
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_op_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Op {}

impl std::hash::Hash for Op {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_op_hash(self.inner) }.hash(state);
    }
}
