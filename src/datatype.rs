use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;

use crate::{Sort, Term};

// ---------------------------------------------------------------------------
// DatatypeConstructorDecl
// ---------------------------------------------------------------------------

/// A declaration for a datatype constructor (before the datatype is resolved).
pub struct DatatypeConstructorDecl {
    pub(crate) inner: Cvc5DatatypeConstructorDecl,
}

impl Clone for DatatypeConstructorDecl {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_dt_cons_decl_copy(self.inner) },
        }
    }
}

impl Drop for DatatypeConstructorDecl {
    fn drop(&mut self) {
        unsafe { cvc5_dt_cons_decl_release(self.inner) }
    }
}

impl DatatypeConstructorDecl {
    pub(crate) fn from_raw(raw: Cvc5DatatypeConstructorDecl) -> Self {
        Self { inner: raw }
    }

    /// Add a selector with the given name and codomain sort.
    pub fn add_selector(&mut self, name: &str, sort: Sort) {
        let c = CString::new(name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector(self.inner, c.as_ptr(), sort.inner) }
    }

    /// Add a selector whose codomain is the datatype itself (self-reference).
    pub fn add_selector_self(&mut self, name: &str) {
        let c = CString::new(name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector_self(self.inner, c.as_ptr()) }
    }

    /// Add a selector whose codomain is an unresolved datatype with the given name.
    pub fn add_selector_unresolved(&mut self, name: &str, unres_name: &str) {
        let n = CString::new(name).unwrap();
        let u = CString::new(unres_name).unwrap();
        unsafe { cvc5_dt_cons_decl_add_selector_unresolved(self.inner, n.as_ptr(), u.as_ptr()) }
    }
}

impl fmt::Display for DatatypeConstructorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_cons_decl_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

impl PartialEq for DatatypeConstructorDecl {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_cons_decl_is_equal(self.inner, other.inner) }
    }
}
impl Eq for DatatypeConstructorDecl {}

impl std::hash::Hash for DatatypeConstructorDecl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_cons_decl_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for DatatypeConstructorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatatypeConstructorDecl({self})")
    }
}

// ---------------------------------------------------------------------------
// DatatypeDecl
// ---------------------------------------------------------------------------

/// A declaration for a datatype (before it is resolved into a sort).
pub struct DatatypeDecl {
    pub(crate) inner: Cvc5DatatypeDecl,
}

impl Clone for DatatypeDecl {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_dt_decl_copy(self.inner) },
        }
    }
}

impl Drop for DatatypeDecl {
    fn drop(&mut self) {
        unsafe { cvc5_dt_decl_release(self.inner) }
    }
}

impl DatatypeDecl {
    pub(crate) fn from_raw(raw: Cvc5DatatypeDecl) -> Self {
        Self { inner: raw }
    }

    /// Create a copy of this declaration (increments the internal reference count).
    pub fn copy(&self) -> DatatypeDecl {
        DatatypeDecl::from_raw(unsafe { cvc5_dt_decl_copy(self.inner) })
    }

    /// Add a constructor declaration to this datatype.
    pub fn add_constructor(&mut self, ctor: &DatatypeConstructorDecl) {
        unsafe { cvc5_dt_decl_add_constructor(self.inner, ctor.inner) }
    }

    /// Get the number of constructors in this declaration.
    pub fn num_constructors(&self) -> usize {
        unsafe { cvc5_dt_decl_get_num_constructors(self.inner) }
    }

    /// Return `true` if this datatype declaration is parametric.
    pub fn is_parametric(&self) -> bool {
        unsafe { cvc5_dt_decl_is_parametric(self.inner) }
    }

    /// Return `true` if this datatype declaration has been resolved.
    pub fn is_resolved(&self) -> bool {
        unsafe { cvc5_dt_decl_is_resolved(self.inner) }
    }

    /// Get the name of this datatype declaration.
    pub fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(cvc5_dt_decl_get_name(self.inner))
                .to_str()
                .unwrap_or("")
        }
    }
}

impl fmt::Display for DatatypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_decl_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

impl PartialEq for DatatypeDecl {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_decl_is_equal(self.inner, other.inner) }
    }
}
impl Eq for DatatypeDecl {}

impl std::hash::Hash for DatatypeDecl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_decl_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for DatatypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatatypeDecl({self})")
    }
}

// ---------------------------------------------------------------------------
// DatatypeSelector
// ---------------------------------------------------------------------------

/// A selector of a resolved datatype constructor.
pub struct DatatypeSelector {
    pub(crate) inner: Cvc5DatatypeSelector,
}

impl Clone for DatatypeSelector {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_dt_sel_copy(self.inner) },
        }
    }
}

impl Drop for DatatypeSelector {
    fn drop(&mut self) {
        unsafe { cvc5_dt_sel_release(self.inner) }
    }
}

impl DatatypeSelector {
    pub(crate) fn from_raw(raw: Cvc5DatatypeSelector) -> Self {
        Self { inner: raw }
    }

    /// Create a copy of this selector (increments the internal reference count).
    pub fn copy(&self) -> DatatypeSelector {
        DatatypeSelector::from_raw(unsafe { cvc5_dt_sel_copy(self.inner) })
    }

    /// Get the name of this selector.
    pub fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(cvc5_dt_sel_get_name(self.inner))
                .to_str()
                .unwrap_or("")
        }
    }

    /// Get the selector function as a term.
    pub fn term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_sel_get_term(self.inner) })
    }

    /// Get the updater function as a term.
    pub fn updater_term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_sel_get_updater_term(self.inner) })
    }

    /// Get the codomain (return) sort of this selector.
    pub fn codomain_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_dt_sel_get_codomain_sort(self.inner) })
    }
}

impl fmt::Display for DatatypeSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_sel_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

impl PartialEq for DatatypeSelector {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_sel_is_equal(self.inner, other.inner) }
    }
}
impl Eq for DatatypeSelector {}

impl std::hash::Hash for DatatypeSelector {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_sel_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for DatatypeSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatatypeSelector({self})")
    }
}

// ---------------------------------------------------------------------------
// DatatypeConstructor
// ---------------------------------------------------------------------------

/// A constructor of a resolved datatype.
pub struct DatatypeConstructor {
    pub(crate) inner: Cvc5DatatypeConstructor,
}

impl Clone for DatatypeConstructor {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_dt_cons_copy(self.inner) },
        }
    }
}

impl Drop for DatatypeConstructor {
    fn drop(&mut self) {
        unsafe { cvc5_dt_cons_release(self.inner) }
    }
}

impl DatatypeConstructor {
    pub(crate) fn from_raw(raw: Cvc5DatatypeConstructor) -> Self {
        Self { inner: raw }
    }

    /// Get the name of this constructor.
    pub fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(cvc5_dt_cons_get_name(self.inner))
                .to_str()
                .unwrap_or("")
        }
    }

    /// Get the constructor function as a term.
    pub fn term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_term(self.inner) })
    }

    /// Get the constructor term instantiated for the given parametric datatype sort.
    pub fn instantiated_term(&self, sort: Sort) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_instantiated_term(self.inner, sort.inner) })
    }

    /// Get the tester (discriminator) function as a term.
    pub fn tester_term(&self) -> Term {
        Term::from_raw(unsafe { cvc5_dt_cons_get_tester_term(self.inner) })
    }

    /// Get the number of selectors of this constructor.
    pub fn num_selectors(&self) -> usize {
        unsafe { cvc5_dt_cons_get_num_selectors(self.inner) }
    }

    /// Get the selector at the given index.
    pub fn selector(&self, index: usize) -> DatatypeSelector {
        DatatypeSelector::from_raw(unsafe { cvc5_dt_cons_get_selector(self.inner, index) })
    }

    /// Get a selector by name.
    pub fn selector_by_name(&self, name: &str) -> DatatypeSelector {
        let c = CString::new(name).unwrap();
        DatatypeSelector::from_raw(unsafe {
            cvc5_dt_cons_get_selector_by_name(self.inner, c.as_ptr())
        })
    }
}

impl fmt::Display for DatatypeConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_cons_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

impl PartialEq for DatatypeConstructor {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_cons_is_equal(self.inner, other.inner) }
    }
}
impl Eq for DatatypeConstructor {}

impl std::hash::Hash for DatatypeConstructor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_cons_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for DatatypeConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatatypeConstructor({self})")
    }
}

// ---------------------------------------------------------------------------
// Datatype
// ---------------------------------------------------------------------------

/// A resolved datatype.
///
/// Obtained from a sort via [`Sort::datatype`] after the datatype has been
/// created with [`TermManager::mk_dt_sort`](crate::TermManager::mk_dt_sort).
pub struct Datatype {
    pub(crate) inner: Cvc5Datatype,
}

impl Clone for Datatype {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_dt_copy(self.inner) },
        }
    }
}

impl Drop for Datatype {
    fn drop(&mut self) {
        unsafe { cvc5_dt_release(self.inner) }
    }
}

impl Datatype {
    pub(crate) fn from_raw(raw: Cvc5Datatype) -> Self {
        Self { inner: raw }
    }

    /// Create a copy of this datatype (increments the internal reference count).
    pub fn copy(&self) -> Datatype {
        Datatype::from_raw(unsafe { cvc5_dt_copy(self.inner) })
    }

    /// Get the constructor at the given index.
    pub fn constructor(&self, index: usize) -> DatatypeConstructor {
        DatatypeConstructor::from_raw(unsafe { cvc5_dt_get_constructor(self.inner, index) })
    }

    /// Get a constructor by name.
    pub fn constructor_by_name(&self, name: &str) -> DatatypeConstructor {
        let c = CString::new(name).unwrap();
        DatatypeConstructor::from_raw(unsafe {
            cvc5_dt_get_constructor_by_name(self.inner, c.as_ptr())
        })
    }

    /// Get a selector by name (searches all constructors).
    pub fn selector(&self, name: &str) -> DatatypeSelector {
        let c = CString::new(name).unwrap();
        DatatypeSelector::from_raw(unsafe { cvc5_dt_get_selector(self.inner, c.as_ptr()) })
    }

    /// Get the name of this datatype.
    pub fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(cvc5_dt_get_name(self.inner))
                .to_str()
                .unwrap_or("")
        }
    }

    /// Get the number of constructors.
    pub fn num_constructors(&self) -> usize {
        unsafe { cvc5_dt_get_num_constructors(self.inner) }
    }

    /// Get the sort parameters of a parametric datatype.
    pub fn parameters(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_dt_get_parameters(self.inner, &mut size) };
        (0..size)
            .map(|i| Sort::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    /// Return `true` if this datatype is parametric.
    pub fn is_parametric(&self) -> bool {
        unsafe { cvc5_dt_is_parametric(self.inner) }
    }

    /// Return `true` if this is a codatatype.
    pub fn is_codatatype(&self) -> bool {
        unsafe { cvc5_dt_is_codatatype(self.inner) }
    }

    /// Return `true` if this is a tuple datatype.
    pub fn is_tuple(&self) -> bool {
        unsafe { cvc5_dt_is_tuple(self.inner) }
    }

    /// Return `true` if this is a record datatype.
    pub fn is_record(&self) -> bool {
        unsafe { cvc5_dt_is_record(self.inner) }
    }

    /// Return `true` if this datatype is finite.
    pub fn is_finite(&self) -> bool {
        unsafe { cvc5_dt_is_finite(self.inner) }
    }

    /// Return `true` if this datatype is well-founded.
    pub fn is_well_founded(&self) -> bool {
        unsafe { cvc5_dt_is_well_founded(self.inner) }
    }
}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_dt_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

impl PartialEq for Datatype {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_dt_is_equal(self.inner, other.inner) }
    }
}
impl Eq for Datatype {}

impl std::hash::Hash for Datatype {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_dt_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Datatype({self})")
    }
}
