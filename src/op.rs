use cvc5_sys::Cvc5Op;

/// A cvc5 operator (indexed operator).
#[derive(Clone, Copy)]
pub struct Op {
    pub(crate) inner: Cvc5Op,
}

impl Op {
    pub(crate) fn from_raw(raw: Cvc5Op) -> Self {
        Self { inner: raw }
    }
}
