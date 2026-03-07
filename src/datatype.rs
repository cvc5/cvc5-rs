use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;

use crate::{Sort, Term};

/// A cvc5 datatype constructor declaration.
pub struct DatatypeConstructorDecl {
    pub(crate) inner: Cvc5DatatypeConstructorDecl,
}

impl DatatypeConstructorDecl {
    pub(crate) fn from_raw(raw: Cvc5DatatypeConstructorDecl) -> Self {
        Self { inner: raw }
    }

    pub fn release(self) { unsafe { cvc5_dt_cons_decl_release(self.inner) } }

    /// Add a selector with the given name and sort.
    pub fn add_selector(&mut self, name: &str, sort: Sort) {
        let c = CString::new(name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector(self.inner, c.as_ptr(), sort.inner) }
    }

    /// Add a selector whose codomain is the datatype itself.
    pub fn add_selector_self(&mut self, name: &str) {
        let c = CString::new(name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector_self(self.inner, c.as_ptr()) }
    }

    /// Add a selector whose codomain is an unresolved datatype.
    pub fn add_selector_unresolved(&mut self, name: &str, unres_name: &str) {
        let n = CString::new(name).unwrap();
        let u = CString::new(unres_name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector_unresolved(self.inner, n.as_ptr(), u.as_ptr()) }
    }
}

impl fmt::Display for DatatypeConstructorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_cons_decl_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for DatatypeConstructorDecl {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_cons_decl_is_equal(self.inner, other.inner) }
    }
}

impl std::hash::Hash for DatatypeConstructorDecl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_cons_decl_hash(self.inner) }.hash(state);
    }
}

/// A cvc5 datatype declaration.
pub struct DatatypeDecl {
    pub(crate) inner: Cvc5DatatypeDecl,
}

impl DatatypeDecl {
    pub(crate) fn from_raw(raw: Cvc5DatatypeDecl) -> Self {
        Self { inner: raw }
    }

    pub fn copy(&self) -> DatatypeDecl { DatatypeDecl::from_raw(unsafe { cvc5_dt_decl_copy(self.inner) }) }
    pub fn release(self) { unsafe { cvc5_dt_decl_release(self.inner) } }

    /// Add a constructor declaration.
    pub fn add_constructor(&mut self, ctor: &DatatypeConstructorDecl) {
        unsafe { cvc5_dt_decl_add_constructor(self.inner, ctor.inner) }
    }

    /// Get the number of constructors.
    pub fn num_constructors(&self) -> usize {
        unsafe { cvc5_dt_decl_get_num_constructors(self.inner) }
    }

    /// Check if parametric.
    pub fn is_parametric(&self) -> bool {
        unsafe { cvc5_dt_decl_is_parametric(self.inner) }
    }

    /// Check if resolved.
    pub fn is_resolved(&self) -> bool {
        unsafe { cvc5_dt_decl_is_resolved(self.inner) }
    }

    /// Get the name.
    pub fn name(&self) -> &str {
        unsafe {
            let s = cvc5_dt_decl_get_name(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }
}

impl fmt::Display for DatatypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_decl_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for DatatypeDecl {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_decl_is_equal(self.inner, other.inner) }
    }
}

impl std::hash::Hash for DatatypeDecl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_decl_hash(self.inner) }.hash(state);
    }
}

/// A cvc5 datatype selector.
#[derive(Clone, Copy)]
pub struct DatatypeSelector {
    pub(crate) inner: Cvc5DatatypeSelector,
}

impl DatatypeSelector {
    pub(crate) fn from_raw(raw: Cvc5DatatypeSelector) -> Self {
        Self { inner: raw }
    }

    pub fn copy(&self) -> DatatypeSelector { DatatypeSelector::from_raw(unsafe { cvc5_dt_sel_copy(self.inner) }) }
    pub fn release(self) { unsafe { cvc5_dt_sel_release(self.inner) } }

    pub fn name(&self) -> &str {
        unsafe {
            let s = cvc5_dt_sel_get_name(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    pub fn term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_sel_get_term(self.inner) })
    }

    pub fn updater_term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_sel_get_updater_term(self.inner) })
    }

    pub fn codomain_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_dt_sel_get_codomain_sort(self.inner) })
    }
}

impl fmt::Display for DatatypeSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_sel_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for DatatypeSelector {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_sel_is_equal(self.inner, other.inner) }
    }
}

impl std::hash::Hash for DatatypeSelector {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_sel_hash(self.inner) }.hash(state);
    }
}

/// A cvc5 datatype constructor.
#[derive(Clone, Copy)]
pub struct DatatypeConstructor {
    pub(crate) inner: Cvc5DatatypeConstructor,
}

impl DatatypeConstructor {
    pub(crate) fn from_raw(raw: Cvc5DatatypeConstructor) -> Self {
        Self { inner: raw }
    }

    pub fn release(self) { unsafe { cvc5_dt_cons_release(self.inner) } }

    pub fn name(&self) -> &str {
        unsafe {
            let s = cvc5_dt_cons_get_name(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    pub fn term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_term(self.inner) })
    }

    pub fn instantiated_term(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_instantiated_term(self.inner, sort.inner) })
    }

    pub fn tester_term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_tester_term(self.inner) })
    }

    pub fn num_selectors(&self) -> usize {
        unsafe { cvc5_dt_cons_get_num_selectors(self.inner) }
    }

    pub fn selector(&self, index: usize) -> DatatypeSelector {
        DatatypeSelector::from_raw(unsafe { cvc5_dt_cons_get_selector(self.inner, index) })
    }

    pub fn selector_by_name(&self, name: &str) -> DatatypeSelector {
        let c = CString::new(name).unwrap();
        DatatypeSelector::from_raw(unsafe { cvc5_dt_cons_get_selector_by_name(self.inner, c.as_ptr()) })
    }
}

impl fmt::Display for DatatypeConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_cons_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for DatatypeConstructor {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_cons_is_equal(self.inner, other.inner) }
    }
}

impl std::hash::Hash for DatatypeConstructor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_cons_hash(self.inner) }.hash(state);
    }
}

/// A cvc5 datatype.
#[derive(Clone, Copy)]
pub struct Datatype {
    pub(crate) inner: Cvc5Datatype,
}

impl Datatype {
    pub(crate) fn from_raw(raw: Cvc5Datatype) -> Self {
        Self { inner: raw }
    }

    pub fn copy(&self) -> Datatype { Datatype::from_raw(unsafe { cvc5_dt_copy(self.inner) }) }
    pub fn release(self) { unsafe { cvc5_dt_release(self.inner) } }

    pub fn constructor(&self, index: usize) -> DatatypeConstructor {
        DatatypeConstructor::from_raw(unsafe { cvc5_dt_get_constructor(self.inner, index) })
    }

    pub fn constructor_by_name(&self, name: &str) -> DatatypeConstructor {
        let c = CString::new(name).unwrap();
        DatatypeConstructor::from_raw(unsafe { cvc5_dt_get_constructor_by_name(self.inner, c.as_ptr()) })
    }

    pub fn selector(&self, name: &str) -> DatatypeSelector {
        let c = CString::new(name).unwrap();
        DatatypeSelector::from_raw(unsafe { cvc5_dt_get_selector(self.inner, c.as_ptr()) })
    }

    pub fn name(&self) -> &str {
        unsafe {
            let s = cvc5_dt_get_name(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    pub fn num_constructors(&self) -> usize {
        unsafe { cvc5_dt_get_num_constructors(self.inner) }
    }

    pub fn parameters(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_dt_get_parameters(self.inner, &mut size) };
        (0..size).map(|i| Sort::from_raw(unsafe { *ptr.add(i) })).collect()
    }

    pub fn is_parametric(&self) -> bool {
        unsafe { cvc5_dt_is_parametric(self.inner) }
    }

    pub fn is_codatatype(&self) -> bool {
        unsafe { cvc5_dt_is_codatatype(self.inner) }
    }

    pub fn is_tuple(&self) -> bool {
        unsafe { cvc5_dt_is_tuple(self.inner) }
    }

    pub fn is_record(&self) -> bool {
        unsafe { cvc5_dt_is_record(self.inner) }
    }

    pub fn is_finite(&self) -> bool {
        unsafe { cvc5_dt_is_finite(self.inner) }
    }

    pub fn is_well_founded(&self) -> bool {
        unsafe { cvc5_dt_is_well_founded(self.inner) }
    }
}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for Datatype {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_is_equal(self.inner, other.inner) }
    }
}

impl std::hash::Hash for Datatype {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_hash(self.inner) }.hash(state);
    }
}
