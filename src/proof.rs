use cvc5_sys::*;
use std::fmt;

use crate::Term;
use crate::error::Result;
use crate::ffi::{checked, non_null, raw_slice};

/// A cvc5 proof object.
///
/// Proofs are produced when the solver is configured with
/// `set_option("produce-proofs", "true")` and a query returns unsat.
pub struct Proof {
    pub(crate) inner: cvc5_sys::Proof,
}

impl Clone for Proof {
    fn clone(&self) -> Self {
        Self::from_raw(self.inner)
    }
}

impl Drop for Proof {
    fn drop(&mut self) {
        unsafe { proof_release(self.inner) }
    }
}

impl Proof {
    /// Wrap a raw proof, taking a reference of our own.
    ///
    /// The single reference on an exported proof belongs to the
    /// [`Solver`](crate::Solver) that made it, and `~Cvc5` drops that reference.
    /// Holding one ourselves is what lets this outlive the solver: the C object
    /// detaches (its solver back-pointer is nulled) instead of being freed.
    pub(crate) fn from_raw(raw: cvc5_sys::Proof) -> Self {
        let inner = non_null(raw, "Proof");
        unsafe { proof_copy(inner) };
        Self { inner }
    }

    /// Get the proof rule used at the root of this proof node.
    pub fn rule(&self) -> cvc5_sys::ProofRule {
        unsafe { proof_get_rule(self.inner) }
    }

    /// Create a copy of this proof (increments the internal reference count).
    pub fn copy(&self) -> Proof {
        Proof::from_raw(self.inner)
    }

    /// Check disequality with another proof.
    pub fn is_disequal(&self, other: &Proof) -> bool {
        unsafe { proof_is_disequal(self.inner, other.inner) }
    }

    /// Get the rewrite rule used at the root of this proof node.
    pub fn rewrite_rule(&self) -> Result<cvc5_sys::ProofRewriteRule> {
        let v = unsafe { proof_get_rewrite_rule(self.inner) };
        checked(v, "rewrite_rule")
    }

    /// Get the conclusion (result) of this proof node as a term.
    pub fn result(&self) -> Term {
        Term::from_raw(unsafe { proof_get_result(self.inner) })
    }

    /// Get the child proof nodes.
    ///
    /// # Known leak on a detached proof
    ///
    /// `cvc5_proof_t::export_proof` is inconsistent about who owns the
    /// reference it returns. While the producing solver is alive it delegates to
    /// `Cvc5::export_proof`, whose result is owned by the *solver*, so the caller
    /// must take a reference of its own, which this wrapper does on
    /// construction. Once that solver is gone it instead returns
    /// `new cvc5_proof_t(nullptr, d_tm, proof)`, a reference owned by the
    /// *caller*, and the extra reference is never dropped.
    ///
    /// Nothing in the C API distinguishes the two cases, so calling `children`
    /// on a proof whose solver has already been dropped leaks that child (and,
    /// through its `d_tm` reference, the term manager). Calling it while the
    /// solver is alive is exact. This is memory-safe either way; the alternative
    /// (never copying) would instead dangle when the solver outlives nothing.
    /// Reported upstream; see the note in `ownership.rs`.
    pub fn children(&self) -> Vec<Proof> {
        let mut size = 0usize;
        let ptr = unsafe { proof_get_children(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Proof::from_raw(p))
            .collect()
    }

    /// Get the arguments of this proof node as terms.
    pub fn arguments(&self) -> Vec<Term> {
        let mut size = 0usize;
        let ptr = unsafe { proof_get_arguments(self.inner, &mut size) };
        unsafe { raw_slice(ptr, size) }
            .iter()
            .map(|&p| Term::from_raw(p))
            .collect()
    }
}

impl PartialEq for Proof {
    fn eq(&self, other: &Self) -> bool {
        unsafe { proof_is_equal(self.inner, other.inner) }
    }
}

impl Eq for Proof {}

impl std::hash::Hash for Proof {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { proof_hash(self.inner) }.hash(state);
    }
}

impl fmt::Debug for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proof({:?})", self.rule())
    }
}
