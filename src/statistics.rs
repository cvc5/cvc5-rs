use crate::error::Result;
use crate::ffi::{checked, cstr_or_empty, cstr_to_string, non_null, raw_slice, wrap};
use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;

/// Solver statistics collected during solving.
///
/// Owns a reference to the [`TermManager`](crate::TermManager) that allocated it
/// (they live in `Cvc5TermManager::d_alloc_statistics`), so it may outlive the
/// `TermManager` binding.
pub struct Statistics {
    pub(crate) inner: cvc5_sys::Statistics,
}

impl Clone for Statistics {
    fn clone(&self) -> Self {
        Self::from_raw(unsafe { stats_copy(self.inner) })
    }
}

impl Drop for Statistics {
    fn drop(&mut self) {
        unsafe { stats_release(self.inner) }
    }
}

impl Statistics {
    pub(crate) fn from_raw(raw: cvc5_sys::Statistics) -> Self {
        Self {
            inner: non_null(raw, "Statistics"),
        }
    }

    /// Look up a statistic by name.
    pub fn get(&self, name: &str) -> Result<Stat> {
        let c = CString::new(name).unwrap();
        let raw = unsafe { stats_get(self.inner, c.as_ptr()) };
        wrap(raw, "get")
    }

    /// Initialize the statistics iterator.
    ///
    /// - `internal` — include internal (non-public) statistics.
    /// - `dflt` — include statistics that still have their default value.
    pub fn iter_init(&mut self, internal: bool, dflt: bool) {
        unsafe { stats_iter_init(self.inner, internal, dflt) }
    }

    /// Return `true` if the iterator has more elements.
    ///
    /// Fails if [`iter_init`](Self::iter_init) has not been called: cvc5 checks
    /// `d_iter != nullptr` and reports "iterator not initialized".
    pub fn iter_has_next(&mut self) -> Result<bool> {
        let v = unsafe { stats_iter_has_next(self.inner) };
        checked(v, "iter_has_next")
    }

    /// Advance the iterator and return the next `(name, stat)` pair.
    ///
    /// Fails if [`iter_init`](Self::iter_init) has not been called, as
    /// [`iter_has_next`](Self::iter_has_next).
    pub fn iter_next(&mut self) -> Result<(String, Stat)> {
        let mut name: *const std::os::raw::c_char = std::ptr::null();
        let s = unsafe { stats_iter_next(self.inner, &mut name) };
        // `name` is only written on success, so gate on the error state before
        // reading it.
        let s = checked(s, "iter_next")?;
        let stat = Stat::from_raw(s);
        let n = unsafe { cstr_to_string(name, "Statistics::iter_next name") };
        Ok((n, stat))
    }

    /// Advance the iterator and return only the next [`Stat`], ignoring the name.
    pub fn iter_next_stat(&mut self) -> Result<Stat> {
        Ok(self.iter_next()?.1)
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statistics({self})")
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { stats_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

/// A single statistic value.
///
/// Owns a reference to the [`TermManager`](crate::TermManager) that allocated
/// it, as [`Statistics`] does.
pub struct Stat {
    pub(crate) inner: cvc5_sys::Stat,
}

impl Clone for Stat {
    fn clone(&self) -> Self {
        Self::from_raw(unsafe { stat_copy(self.inner) })
    }
}

impl Drop for Stat {
    fn drop(&mut self) {
        unsafe { stat_release(self.inner) }
    }
}

impl Stat {
    pub(crate) fn from_raw(raw: cvc5_sys::Stat) -> Self {
        Self {
            inner: non_null(raw, "Stat"),
        }
    }

    /// Return `true` if this is an internal (non-public) statistic.
    pub fn is_internal(&self) -> bool {
        unsafe { stat_is_internal(self.inner) }
    }

    /// Return `true` if this statistic still has its default value.
    pub fn is_default(&self) -> bool {
        unsafe { stat_is_default(self.inner) }
    }

    /// Return `true` if this statistic holds an integer value.
    pub fn is_int(&self) -> bool {
        unsafe { stat_is_int(self.inner) }
    }

    /// Return `true` if this statistic holds a double value.
    pub fn is_double(&self) -> bool {
        unsafe { stat_is_double(self.inner) }
    }

    /// Return `true` if this statistic holds a string value.
    pub fn is_string(&self) -> bool {
        unsafe { stat_is_string(self.inner) }
    }

    /// Return `true` if this statistic holds a histogram.
    pub fn is_histogram(&self) -> bool {
        unsafe { stat_is_histogram(self.inner) }
    }

    /// Get the integer value of this statistic.
    pub fn get_int(&self) -> Result<i64> {
        let v = unsafe { stat_get_int(self.inner) };
        checked(v, "get_int")
    }

    /// Get the double value of this statistic.
    pub fn get_double(&self) -> Result<f64> {
        let v = unsafe { stat_get_double(self.inner) };
        checked(v, "get_double")
    }

    /// Get the string value of this statistic.
    pub fn get_string(&self) -> Result<String> {
        let p = unsafe { stat_get_string(self.inner) };
        let p = checked(p, "get_string")?;
        Ok(unsafe { cstr_or_empty(p) }.to_owned())
    }

    /// Get the histogram value as a list of `(key, count)` pairs.
    pub fn get_histogram(&self) -> Result<Vec<(String, u64)>> {
        let mut keys: *mut *const std::os::raw::c_char = std::ptr::null_mut();
        let mut values: *mut u64 = std::ptr::null_mut();
        let mut size = 0usize;
        unsafe { stat_get_histogram(self.inner, &mut keys, &mut values, &mut size) };
        checked((), "get_histogram")?;
        let keys = unsafe { raw_slice(keys, size) };
        let values = unsafe { raw_slice(values, size) };
        Ok(keys
            .iter()
            .zip(values)
            .map(|(&k, &v)| {
                let k = unsafe { cstr_to_string(k, "Stat::get_histogram key") };
                (k, v)
            })
            .collect())
    }
}

impl fmt::Debug for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stat({self})")
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { stat_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}
