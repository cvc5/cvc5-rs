use cvc5_sys::*;
use std::fmt;

use crate::Term;

/// A cvc5 proof.
pub struct Proof {
    pub(crate) inner: Cvc5Proof,
}

impl Clone for Proof {
    fn clone(&self) -> Self {
        Self {
            inner: unsafe { cvc5_proof_copy(self.inner) },
        }
    }
}

impl Drop for Proof {
    fn drop(&mut self) {
        unsafe { cvc5_proof_release(self.inner) }
    }
}

impl Proof {
    pub(crate) fn from_raw(raw: Cvc5Proof) -> Self {
        Self { inner: raw }
    }

    pub fn rule(&self) -> Cvc5ProofRule {
        unsafe { cvc5_proof_get_rule(self.inner) }
    }

    pub fn copy(&self) -> Proof {
        Proof::from_raw(unsafe { cvc5_proof_copy(self.inner) })
    }
    pub fn release(self) {
        unsafe { cvc5_proof_release(self.inner) }
    }
    pub fn is_disequal(&self, other: &Proof) -> bool {
        unsafe { cvc5_proof_is_disequal(self.inner, other.inner) }
    }

    pub fn rewrite_rule(&self) -> Cvc5ProofRewriteRule {
        unsafe { cvc5_proof_get_rewrite_rule(self.inner) }
    }

    pub fn result(&self) -> Term {
        Term::from_raw(unsafe { cvc5_proof_get_result(self.inner) })
    }

    pub fn children(&self) -> Vec<Proof> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_proof_get_children(self.inner, &mut size) };
        (0..size)
            .map(|i| Proof::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }

    pub fn arguments(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { cvc5_proof_get_arguments(self.inner, &mut size) };
        (0..size)
            .map(|i| Term::from_raw(unsafe { *ptr.add(i) }))
            .collect()
    }
}

impl PartialEq for Proof {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cvc5_proof_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Proof {}

impl std::hash::Hash for Proof {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { cvc5_proof_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proof({:?})", self.rule())
    }
}
