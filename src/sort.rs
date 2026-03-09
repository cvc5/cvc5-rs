use cvc5_sys::*;
use std::fmt;

use crate::Datatype;

/// A cvc5 sort (type).
///
/// Sorts represent types in the SMT solver — Boolean, Integer, Real,
/// BitVector, Array, Datatype, etc.
pub struct Sort {
    pub(crate) inner: Cvc5Sort,
}

impl Clone for Sort {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_sort_copy(self.inner) },
        }
    }
}

impl Drop for Sort {
    fn drop(&mut self) {
        unsafe { cvc5_sort_release(self.inner) }
    }
}

impl Sort {
    pub(crate) fn from_raw(raw: Cvc5Sort) -> Self {
        Self { inner: raw }
    }

    /// Get the kind of this sort.
    pub fn kind(&self) -> Cvc5SortKind {
        unsafe { cvc5_sort_get_kind(self.inner) }
    }

    /// Create a copy of this sort (increments the internal reference count).
    pub fn copy(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_copy(self.inner) })
    }

    /// Check disequality with another sort.
    pub fn is_disequal(&self, other: &Sort) -> bool {
        unsafe { cvc5_sort_is_disequal(self.inner, other.inner) }
    }

    /// Return `true` if this sort has a symbol (name).
    pub fn has_symbol(&self) -> bool {
        unsafe { cvc5_sort_has_symbol(self.inner) }
    }

    /// Get the symbol (name) of this sort.
    pub fn symbol(&self) -> &str {
        unsafe {
            let s = cvc5_sort_get_symbol(self.inner);
            std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
        }
    }

    /// Return `true` if this is the Boolean sort.
    pub fn is_boolean(&self) -> bool {
        unsafe { cvc5_sort_is_boolean(self.inner) }
    }
    /// Return `true` if this is the Integer sort.
    pub fn is_integer(&self) -> bool {
        unsafe { cvc5_sort_is_integer(self.inner) }
    }
    /// Return `true` if this is the Real sort.
    pub fn is_real(&self) -> bool {
        unsafe { cvc5_sort_is_real(self.inner) }
    }
    /// Return `true` if this is the String sort.
    pub fn is_string(&self) -> bool {
        unsafe { cvc5_sort_is_string(self.inner) }
    }
    /// Return `true` if this is the RegExp sort.
    pub fn is_regexp(&self) -> bool {
        unsafe { cvc5_sort_is_regexp(self.inner) }
    }
    /// Return `true` if this is the rounding mode sort.
    pub fn is_rm(&self) -> bool {
        unsafe { cvc5_sort_is_rm(self.inner) }
    }
    /// Return `true` if this is a bit-vector sort.
    pub fn is_bv(&self) -> bool {
        unsafe { cvc5_sort_is_bv(self.inner) }
    }
    /// Return `true` if this is a floating-point sort.
    pub fn is_fp(&self) -> bool {
        unsafe { cvc5_sort_is_fp(self.inner) }
    }
    /// Return `true` if this is a datatype sort.
    pub fn is_dt(&self) -> bool {
        unsafe { cvc5_sort_is_dt(self.inner) }
    }
    /// Return `true` if this is a datatype constructor sort.
    pub fn is_dt_constructor(&self) -> bool {
        unsafe { cvc5_sort_is_dt_constructor(self.inner) }
    }
    /// Return `true` if this is a datatype selector sort.
    pub fn is_dt_selector(&self) -> bool {
        unsafe { cvc5_sort_is_dt_selector(self.inner) }
    }
    /// Return `true` if this is a datatype tester sort.
    pub fn is_dt_tester(&self) -> bool {
        unsafe { cvc5_sort_is_dt_tester(self.inner) }
    }
    /// Return `true` if this is a datatype updater sort.
    pub fn is_dt_updater(&self) -> bool {
        unsafe { cvc5_sort_is_dt_updater(self.inner) }
    }
    /// Return `true` if this is a function sort.
    pub fn is_fun(&self) -> bool {
        unsafe { cvc5_sort_is_fun(self.inner) }
    }
    /// Return `true` if this is a predicate sort.
    pub fn is_predicate(&self) -> bool {
        unsafe { cvc5_sort_is_predicate(self.inner) }
    }
    /// Return `true` if this is a tuple sort.
    pub fn is_tuple(&self) -> bool {
        unsafe { cvc5_sort_is_tuple(self.inner) }
    }
    /// Return `true` if this is a nullable sort.
    pub fn is_nullable(&self) -> bool {
        unsafe { cvc5_sort_is_nullable(self.inner) }
    }
    /// Return `true` if this is a record sort.
    pub fn is_record(&self) -> bool {
        unsafe { cvc5_sort_is_record(self.inner) }
    }
    /// Return `true` if this is an array sort.
    pub fn is_array(&self) -> bool {
        unsafe { cvc5_sort_is_array(self.inner) }
    }
    /// Return `true` if this is a finite field sort.
    pub fn is_ff(&self) -> bool {
        unsafe { cvc5_sort_is_ff(self.inner) }
    }
    /// Return `true` if this is a set sort.
    pub fn is_set(&self) -> bool {
        unsafe { cvc5_sort_is_set(self.inner) }
    }
    /// Return `true` if this is a bag sort.
    pub fn is_bag(&self) -> bool {
        unsafe { cvc5_sort_is_bag(self.inner) }
    }
    /// Return `true` if this is a sequence sort.
    pub fn is_sequence(&self) -> bool {
        unsafe { cvc5_sort_is_sequence(self.inner) }
    }
    /// Return `true` if this is an abstract sort.
    pub fn is_abstract(&self) -> bool {
        unsafe { cvc5_sort_is_abstract(self.inner) }
    }
    /// Return `true` if this is an uninterpreted sort.
    pub fn is_uninterpreted_sort(&self) -> bool {
        unsafe { cvc5_sort_is_uninterpreted_sort(self.inner) }
    }
    /// Return `true` if this is an uninterpreted sort constructor.
    pub fn is_uninterpreted_sort_constructor(&self) -> bool {
        unsafe { cvc5_sort_is_uninterpreted_sort_constructor(self.inner) }
    }
    /// Return `true` if this sort is an instantiated (parametric) sort.
    pub fn is_instantiated(&self) -> bool {
        unsafe { cvc5_sort_is_instantiated(self.inner) }
    }

    /// Get the associated uninterpreted sort constructor of an instantiated sort.
    pub fn uninterpreted_sort_constructor(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_get_uninterpreted_sort_constructor(self.inner) })
    }

    /// Get the datatype associated with a datatype sort.
    pub fn datatype(&self) -> Datatype {
        Datatype::from_raw(unsafe { cvc5_sort_get_datatype(self.inner) })
    }

    /// Instantiate a parametric sort with the given sort parameters.
    pub fn instantiate(&self, params: &[Sort]) -> Sort {
        let raw: Vec<Cvc5Sort> = params.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe { cvc5_sort_instantiate(self.inner, raw.len(), raw.as_ptr()) })
    }

    /// Get the sort parameters of an instantiated sort.
    pub fn instantiated_parameters(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_sort_get_instantiated_parameters(self.inner, &mut size) };
        (0..size)
            .map(|i| Sort::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    /// Substitute `s` with `replacement` in this sort.
    pub fn substitute(&self, s: Sort, replacement: Sort) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_substitute(self.inner, s.inner, replacement.inner) })
    }

    /// Simultaneously substitute `sorts` with `replacements` in this sort.
    pub fn substitute_sorts(&self, sorts: &[Sort], replacements: &[Sort]) -> Sort {
        let s: Vec<Cvc5Sort> = sorts.iter().map(|s| s.inner).collect();
        let r: Vec<Cvc5Sort> = replacements.iter().map(|s| s.inner).collect();
        Sort::from_raw(unsafe {
            cvc5_sort_substitute_sorts(self.inner, s.len(), s.as_ptr(), r.as_ptr())
        })
    }

    // -- Datatype constructor sort accessors --

    /// Get the arity of a datatype constructor sort.
    pub fn dt_constructor_arity(&self) -> usize {
        unsafe { cvc5_sort_dt_constructor_get_arity(self.inner) }
    }

    /// Get the domain sorts of a datatype constructor sort.
    pub fn dt_constructor_domain(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_sort_dt_constructor_get_domain(self.inner, &mut size) };
        (0..size)
            .map(|i| Sort::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    /// Get the codomain sort of a datatype constructor sort.
    pub fn dt_constructor_codomain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_dt_constructor_get_codomain(self.inner) })
    }

    // -- Datatype selector sort accessors --

    /// Get the domain sort of a datatype selector sort.
    pub fn dt_selector_domain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_dt_selector_get_domain(self.inner) })
    }

    /// Get the codomain sort of a datatype selector sort.
    pub fn dt_selector_codomain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_dt_selector_get_codomain(self.inner) })
    }

    // -- Datatype tester sort accessors --

    /// Get the domain sort of a datatype tester sort.
    pub fn dt_tester_domain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_dt_tester_get_domain(self.inner) })
    }

    /// Get the codomain sort of a datatype tester sort.
    pub fn dt_tester_codomain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_dt_tester_get_codomain(self.inner) })
    }

    // -- Function sort accessors --

    /// Get the arity of a function sort.
    pub fn fun_arity(&self) -> usize {
        unsafe { cvc5_sort_fun_get_arity(self.inner) }
    }

    /// Get the domain sorts of a function sort.
    pub fn fun_domain(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_sort_fun_get_domain(self.inner, &mut size) };
        (0..size)
            .map(|i| Sort::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    /// Get the codomain sort of a function sort.
    pub fn fun_codomain(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_fun_get_codomain(self.inner) })
    }

    // -- Array sort accessors --

    /// Get the index sort of an array sort.
    pub fn array_index_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_array_get_index_sort(self.inner) })
    }

    /// Get the element sort of an array sort.
    pub fn array_element_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_array_get_element_sort(self.inner) })
    }

    // -- Set / Bag / Sequence element sort --

    /// Get the element sort of a set sort.
    pub fn set_element_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_set_get_element_sort(self.inner) })
    }

    /// Get the element sort of a bag sort.
    pub fn bag_element_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_bag_get_element_sort(self.inner) })
    }

    /// Get the element sort of a sequence sort.
    pub fn sequence_element_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_sequence_get_element_sort(self.inner) })
    }

    // -- Abstract sort --

    /// Get the kind of an abstract sort.
    pub fn abstract_kind(&self) -> Cvc5SortKind {
        unsafe { cvc5_sort_abstract_get_kind(self.inner) }
    }

    // -- Uninterpreted sort constructor --

    /// Get the arity of an uninterpreted sort constructor.
    pub fn uninterpreted_sort_constructor_arity(&self) -> usize {
        unsafe { cvc5_sort_uninterpreted_sort_constructor_get_arity(self.inner) }
    }

    // -- Bit-vector sort --

    /// Get the bit-width of a bit-vector sort.
    pub fn bv_size(&self) -> u32 {
        unsafe { cvc5_sort_bv_get_size(self.inner) }
    }

    // -- Finite field sort --

    /// Get the size (modulus) of a finite field sort as a string.
    pub fn ff_size(&self) -> String {
        unsafe {
            let s = cvc5_sort_ff_get_size(self.inner);
            std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
        }
    }

    // -- Floating-point sort --

    /// Get the exponent size of a floating-point sort.
    pub fn fp_exponent_size(&self) -> u32 {
        unsafe { cvc5_sort_fp_get_exp_size(self.inner) }
    }

    /// Get the significand size of a floating-point sort.
    pub fn fp_significand_size(&self) -> u32 {
        unsafe { cvc5_sort_fp_get_sig_size(self.inner) }
    }

    // -- Datatype sort --

    /// Get the arity of a datatype sort.
    pub fn dt_arity(&self) -> usize {
        unsafe { cvc5_sort_dt_get_arity(self.inner) }
    }

    // -- Tuple sort --

    /// Get the length (number of elements) of a tuple sort.
    pub fn tuple_length(&self) -> usize {
        unsafe { cvc5_sort_tuple_get_length(self.inner) }
    }

    /// Get the element sorts of a tuple sort.
    pub fn tuple_element_sorts(&self) -> Vec<Sort> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_sort_tuple_get_element_sorts(self.inner, &mut size) };
        (0..size)
            .map(|i| Sort::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    // -- Nullable sort --

    /// Get the element sort of a nullable sort.
    pub fn nullable_element_sort(&self) -> Sort {
        Sort::from_raw(unsafe { cvc5_sort_nullable_get_element_sort(self.inner) })
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

impl PartialEq for Sort {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_sort_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Sort {}

impl PartialOrd for Sort {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sort {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let c = unsafe { cvc5_sort_compare(self.inner, other.inner) };
        c.cmp(&0)
    }
}

impl std::hash::Hash for Sort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_sort_hash(self.inner) }.hash(state);
    }
}
