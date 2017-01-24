//  Copyright (C) 2016 Sebastian Dröge <sebastian@centricular.com>
//
//  This library is free software; you can redistribute it and/or
//  modify it under the terms of the GNU Library General Public
//  License as published by the Free Software Foundation; either
//  version 2 of the License, or (at your option) any later version.
//
//  This library is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  Library General Public License for more details.
//
//  You should have received a copy of the GNU Library General Public
//  License along with this library; if not, write to the
//  Free Software Foundation, Inc., 51 Franklin St, Fifth Floor,
//  Boston, MA 02110-1301, USA.
//
//
use libc::c_char;
use std::os::raw::c_void;
use std::ffi::CString;
use std::i32;
use num_rational::Rational32;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GstFlowReturn {
    Ok = 0,
    NotLinked = -1,
    Flushing = -2,
    Eos = -3,
    NotNegotiated = -4,
    Error = -5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GBoolean {
    False = 0,
    True = 1,
}

impl GBoolean {
    pub fn from_bool(v: bool) -> GBoolean {
        if v { GBoolean::True } else { GBoolean::False }
    }

    pub fn to_bool(&self) -> bool {
        !(*self == GBoolean::False)
    }
}

pub struct Element(*const c_void);

impl Element {
    pub unsafe fn new(element: *const c_void) -> Element {
        extern "C" {
            fn gst_object_ref(object: *const c_void) -> *const c_void;
        }

        if element.is_null() {
            panic!("NULL not allowed");
        }

        gst_object_ref(element);

        Element(element)
    }

    pub unsafe fn as_ptr(&self) -> *const c_void {
        self.0
    }
}

impl Drop for Element {
    fn drop(&mut self) {
        extern "C" {
            fn gst_object_unref(object: *const c_void);
        }

        unsafe {
            gst_object_unref(self.0);
        }
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        unsafe { Element::new(self.0) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cstring_drop(ptr: *mut c_char) {
    let _ = CString::from_raw(ptr);
}

pub fn f64_to_fraction(val: f64) -> Option<Rational32> {
    // Continued fractions algorithm
    // http://mathforum.org/dr.math/faq/faq.fractions.html#decfrac

    let negative = val < 0.0;

    let mut q = val.abs();
    let mut n0 = 0;
    let mut d0 = 1;
    let mut n1 = 1;
    let mut d1 = 0;

    const MAX_ITERATIONS: usize = 30;
    const MAX_ERROR: f64 = 1.0e-20;
    // 1/EPSILON > i32::MAX
    const EPSILON: f64 = 1.0e-10;

    // Overflow
    if q > i32::MAX as f64 {
        return None;
    }

    for _ in 0..MAX_ITERATIONS {
        let a = q as u32;
        let f = q - (a as f64);

        // Prevent overflow
        if a != 0 &&
           (n1 > (i32::MAX as u32) / a || d1 > (i32::MAX as u32) / a ||
            a * n1 > (i32::MAX as u32) - n0 || a * d1 > (i32::MAX as u32) - d0) {
            break;
        }

        let n = a * n1 + n0;
        let d = a * d1 + d0;

        n0 = n1;
        d0 = d1;
        n1 = n;
        d1 = d;

        // Prevent division by ~0
        if f < EPSILON {
            break;
        }
        let r = 1.0 / f;

        // Simplify fraction. Doing so here instead of at the end
        // allows us to get closer to the target value without overflows
        let g = gcd(n1, d1);
        if g != 0 {
            n1 /= g;
            d1 /= g;
        }

        // Close enough?
        if ((n as f64) / (d as f64) - val).abs() < MAX_ERROR {
            break;
        }

        q = r;
    }

    // Guaranteed by the overflow check
    assert!(n1 <= i32::MAX as u32);
    assert!(d1 <= i32::MAX as u32);

    // Overflow
    if d1 == 0 {
        return None;
    }

    // Make negative again if needed
    if negative {
        Some(Rational32::new(-(n1 as i32), d1 as i32))
    } else {
        Some(Rational32::new(n1 as i32, d1 as i32))
    }
}

pub fn gcd(mut a: u32, mut b: u32) -> u32 {
    // Euclid's algorithm
    while b != 0 {
        let tmp = a;
        a = b;
        b = tmp % b;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_rational::Rational32;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(2 * 2 * 2 * 2, 2 * 2 * 2 * 3), 2 * 2 * 2);
        assert_eq!(gcd(2 * 3 * 5 * 5 * 7, 2 * 5 * 7), 2 * 5 * 7);
    }

    #[test]
    fn test_f64_to_fraction() {
        assert_eq!(f64_to_fraction(2.0), Some(Rational32::new(2, 1)));
        assert_eq!(f64_to_fraction(2.5), Some(Rational32::new(5, 2)));
        assert_eq!(f64_to_fraction(0.127659574),
                   Some(Rational32::new(29013539, 227272723)));
        assert_eq!(f64_to_fraction(29.97), Some(Rational32::new(2997, 100)));
    }
}