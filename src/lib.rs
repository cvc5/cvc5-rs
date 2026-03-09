//! # cvc5-rs
//!
//! Safe, high-level Rust bindings for the [cvc5](https://cvc5.github.io/) SMT solver.
//!
//! # Example
//!
//! ```no_run
//! use cvc5_rs::{TermManager, Solver, Kind};
//!
//! let tm = TermManager::new();
//! let mut solver = Solver::new(&tm);
//!
//! solver.set_logic("QF_LIA");
//! solver.set_option("produce-models", "true");
//!
//! let int_sort = tm.integer_sort();
//! let x = tm.mk_const(int_sort, "x");
//! let zero = tm.mk_integer(0);
//!
//! let gt = tm.mk_term(Kind::CVC5_KIND_GT, &[x.clone(), zero]);
//! solver.assert_formula(gt);
//!
//! let result = solver.check_sat();
//! assert!(result.is_sat());
//!
//! let x_val = solver.get_value(x);
//! println!("x = {x_val}");
//! ```

mod datatype;
mod grammar;
mod op;
#[cfg(feature = "parser")]
mod parser;
mod proof;
mod result;
mod solver;
mod sort;
mod statistics;
mod synth_result;
mod term;
mod term_manager;

pub use cvc5_sys::Cvc5Kind as Kind;
pub use cvc5_sys::Cvc5OptionInfo;
pub use cvc5_sys::Cvc5Plugin;
pub use cvc5_sys::Cvc5ProofRewriteRule as ProofRewriteRule;
pub use cvc5_sys::Cvc5ProofRule as ProofRule;
pub use cvc5_sys::Cvc5RoundingMode as RoundingMode;
pub use cvc5_sys::Cvc5SkolemId as SkolemId;
pub use cvc5_sys::Cvc5SortKind as SortKind;
pub use cvc5_sys::Cvc5UnknownExplanation as UnknownExplanation;

pub use datatype::{
    Datatype, DatatypeConstructor, DatatypeConstructorDecl, DatatypeDecl, DatatypeSelector,
};
pub use grammar::Grammar;
pub use op::Op;
pub use proof::Proof;
pub use result::Result;
pub use solver::Solver;
pub use sort::Sort;
pub use statistics::{Stat, Statistics};
pub use synth_result::SynthResult;
pub use term::Term;
pub use term_manager::TermManager;

#[cfg(feature = "parser")]
pub use cvc5_sys::Cvc5InputLanguage as InputLanguage;
#[cfg(feature = "parser")]
pub use parser::{Command, InputParser, SymbolManager};
