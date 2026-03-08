use cvc5_sys::*;
use std::fmt;

use crate::Term;

/// A cvc5 grammar for SyGuS.
pub struct Grammar {
    pub(crate) inner: Cvc5Grammar,
}

impl Clone for Grammar {
    fn clone(&self) -> Self { Self { inner: unsafe { cvc5_grammar_copy(self.inner) } } }
}

impl Drop for Grammar {
    fn drop(&mut self) { unsafe { cvc5_grammar_release(self.inner) } }
}

impl Grammar {
    pub(crate) fn from_raw(raw: Cvc5Grammar) -> Self {
        Self { inner: raw }
    }

    pub fn copy(&self) -> Grammar { Grammar::from_raw(unsafe { cvc5_grammar_copy(self.inner) }) }
    pub fn release(self) { unsafe { cvc5_grammar_release(self.inner) } }
    pub fn is_disequal(&self, other: &Grammar) -> bool { unsafe { cvc5_grammar_is_disequal(self.inner, other.inner) } }

    /// Add a rule to the given non-terminal symbol.
    pub fn add_rule(&mut self, symbol: Term, rule: Term) {
        unsafe { cvc5_grammar_add_rule(self.inner, symbol.inner, rule.inner) }
    }

    /// Add rules to the given non-terminal symbol.
    pub fn add_rules(&mut self, symbol: Term, rules: &[Term]) {
        let raw: Vec<Cvc5Term> = rules.iter().map(|t| t.inner).collect();
        unsafe { cvc5_grammar_add_rules(self.inner, symbol.inner, raw.len(), raw.as_ptr()) }
    }

    /// Allow the symbol to be an arbitrary constant.
    pub fn add_any_constant(&mut self, symbol: Term) {
        unsafe { cvc5_grammar_add_any_constant(self.inner, symbol.inner) }
    }

    /// Allow the symbol to be any input variable.
    pub fn add_any_variable(&mut self, symbol: Term) {
        unsafe { cvc5_grammar_add_any_variable(self.inner, symbol.inner) }
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_grammar_to_string(self.inner) };
        let cs = unsafe { std::ffi::CStr::from_ptr(s) };
        write!(f, "{}", cs.to_string_lossy())
    }
}

impl PartialEq for Grammar {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_grammar_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Grammar {}

impl std::hash::Hash for Grammar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_grammar_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Grammar({self})")
    }
}
