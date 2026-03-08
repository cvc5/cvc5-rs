use cvc5_sys::*;
use std::ffi::CString;
use std::fmt;

/// A cvc5 statistics object.
pub struct Statistics {
    pub(crate) inner: Cvc5Statistics,
}

impl Statistics {
    pub(crate) fn from_raw(raw: Cvc5Statistics) -> Self {
        Self { inner: raw }
    }

    pub fn get(&self, name: &str) -> Stat {
        let c = CString::new(name).unwrap();
        Stat {
            inner: unsafe { cvc5_stats_get(self.inner, c.as_ptr()) },
        }
    }

    pub fn iter_init(&self, internal: bool, dflt: bool) {
        unsafe { cvc5_stats_iter_init(self.inner, internal, dflt) }
    }

    pub fn iter_has_next(&self) -> bool {
        unsafe { cvc5_stats_iter_has_next(self.inner) }
    }

    pub fn iter_next(&self) -> (String, Stat) {
        let mut name: *const std::os::raw::c_char = std::ptr::null();
        let s = unsafe { cvc5_stats_iter_next(self.inner, &mut name) };
        let n = unsafe {
            std::ffi::CStr::from_ptr(name)
                .to_string_lossy()
                .into_owned()
        };
        (n, Stat { inner: s })
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Statistics({self})")
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_stats_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}

/// A single statistic value.
pub struct Stat {
    pub(crate) inner: Cvc5Stat,
}

impl Stat {
    pub fn is_internal(&self) -> bool {
        unsafe { cvc5_stat_is_internal(self.inner) }
    }
    pub fn is_default(&self) -> bool {
        unsafe { cvc5_stat_is_default(self.inner) }
    }
    pub fn is_int(&self) -> bool {
        unsafe { cvc5_stat_is_int(self.inner) }
    }
    pub fn is_double(&self) -> bool {
        unsafe { cvc5_stat_is_double(self.inner) }
    }
    pub fn is_string(&self) -> bool {
        unsafe { cvc5_stat_is_string(self.inner) }
    }
    pub fn is_histogram(&self) -> bool {
        unsafe { cvc5_stat_is_histogram(self.inner) }
    }

    pub fn get_int(&self) -> i64 {
        unsafe { cvc5_stat_get_int(self.inner) }
    }
    pub fn get_double(&self) -> f64 {
        unsafe { cvc5_stat_get_double(self.inner) }
    }

    pub fn get_string(&self) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(cvc5_stat_get_string(self.inner))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn get_histogram(&self) -> Vec<(String, u64)> {
        let mut keys: *mut *const std::os::raw::c_char = std::ptr::null_mut();
        let mut values: *mut u64 = std::ptr::null_mut();
        let mut size = 0usize;
        unsafe { cvc5_stat_get_histogram(self.inner, &mut keys, &mut values, &mut size) };
        (0..size)
            .map(|i| unsafe {
                let k = std::ffi::CStr::from_ptr(*keys.add(i))
                    .to_string_lossy()
                    .into_owned();
                let v = *values.add(i);
                (k, v)
            })
            .collect()
    }
}

impl fmt::Debug for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stat({self})")
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { cvc5_stat_to_string(self.inner) };
        write!(f, "{}", unsafe {
            std::ffi::CStr::from_ptr(s).to_string_lossy()
        })
    }
}
