use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;

/// Solver statistics collected during solving.
///
/// The lifetime parameter is bound to the [`TermManager`](crate::TermManager)
/// that owns the statistics: they live in `Cvc5TermManager::d_alloc_statistics`,
/// which `cvc5_term_manager_delete` frees.
pub struct Statistics<'tm> {
    pub(crate) inner: cvc5_sys::Statistics,
    pub(crate) _phantom: PhantomData<&'tm ()>,
}

impl<'tm> Statistics<'tm> {
    pub(crate) fn from_raw(raw: cvc5_sys::Statistics) -> Self {
        Self {
            inner: crate::ffi::non_null(raw, "Statistics"),
            _phantom: PhantomData,
        }
    }

    /// Look up a statistic by name.
    ///
    /// # Safety
    ///
    /// The returned [`Stat`] points into a `std::vector` owned by the term
    /// manager, and the C API hands out a pointer to its last element. Any
    /// further call to [`get`](Self::get), [`iter_next`](Self::iter_next) or
    /// [`iter_next_stat`](Self::iter_next_stat) may grow that vector and
    /// invalidate every `Stat` handed out earlier — the second such call is
    /// already enough. The caller must finish using one `Stat` before obtaining
    /// the next.
    ///
    /// Tracked upstream as cvc5/cvc5#12898; this becomes safe once those arenas
    /// use `std::deque`.
    pub unsafe fn get(&self, name: &str) -> Stat<'tm> {
        let c = CString::new(name).unwrap();
        Stat::from_raw(unsafe { stats_get(self.inner, c.as_ptr()) })
    }

    /// Initialize the statistics iterator.
    ///
    /// - `internal` — include internal (non-public) statistics.
    /// - `dflt` — include statistics that still have their default value.
    pub fn iter_init(&self, internal: bool, dflt: bool) {
        unsafe { stats_iter_init(self.inner, internal, dflt) }
    }

    /// Return `true` if the iterator has more elements.
    pub fn iter_has_next(&self) -> bool {
        unsafe { stats_iter_has_next(self.inner) }
    }

    /// Advance the iterator and return the next `(name, stat)` pair.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get): each call may invalidate every previously returned
    /// [`Stat`], so they cannot be collected. Read what you need from one before
    /// advancing. Tracked upstream as cvc5/cvc5#12898.
    pub unsafe fn iter_next(&self) -> (String, Stat<'tm>) {
        let mut name: *const std::os::raw::c_char = std::ptr::null();
        let s = unsafe { stats_iter_next(self.inner, &mut name) };
        // `name` is only written on success, so guard the stat pointer first:
        // it gives the better panic message when the iterator is exhausted or
        // uninitialized.
        let stat = Stat::from_raw(s);
        let n = unsafe { crate::ffi::cstr_to_string(name, "Statistics::iter_next name") };
        (n, stat)
    }

    /// Advance the iterator and return only the next [`Stat`], ignoring the name.
    ///
    /// # Safety
    ///
    /// As [`iter_next`](Self::iter_next). Tracked upstream as cvc5/cvc5#12898.
    pub unsafe fn iter_next_stat(&self) -> Stat<'tm> {
        unsafe { self.iter_next() }.1
    }
}

impl fmt::Debug for Statistics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statistics({self})")
    }
}

impl fmt::Display for Statistics<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { stats_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

/// A single statistic value.
///
/// Bound to the [`TermManager`](crate::TermManager) that owns it, as
/// [`Statistics`] is.
pub struct Stat<'tm> {
    pub(crate) inner: cvc5_sys::Stat,
    pub(crate) _phantom: PhantomData<&'tm ()>,
}

impl<'tm> Stat<'tm> {
    pub(crate) fn from_raw(raw: cvc5_sys::Stat) -> Self {
        Self {
            inner: crate::ffi::non_null(raw, "Stat"),
            _phantom: PhantomData,
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
    pub fn get_int(&self) -> i64 {
        unsafe { stat_get_int(self.inner) }
    }

    /// Get the double value of this statistic.
    pub fn get_double(&self) -> f64 {
        unsafe { stat_get_double(self.inner) }
    }

    /// Get the string value of this statistic.
    pub fn get_string(&self) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(stat_get_string(self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Get the histogram value as a list of `(key, count)` pairs.
    pub fn get_histogram(&self) -> Vec<(String, u64)> {
        let mut keys: *mut *const std::os::raw::c_char = std::ptr::null_mut();
        let mut values: *mut u64 = std::ptr::null_mut();
        let mut size = 0usize;
        unsafe { stat_get_histogram(self.inner, &mut keys, &mut values, &mut size) };
        // `keys`/`values`/`size` are only written on success; on failure they
        // keep the null/zero initializers above.
        let keys = unsafe { crate::ffi::raw_slice(keys, size) };
        let values = unsafe { crate::ffi::raw_slice(values, size) };
        keys.iter()
            .zip(values)
            .map(|(&k, &v)| {
                let k = unsafe { crate::ffi::cstr_to_string(k, "Stat::get_histogram key") };
                (k, v)
            })
            .collect()
    }
}

impl fmt::Debug for Stat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stat({self})")
    }
}

impl fmt::Display for Stat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { stat_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}
