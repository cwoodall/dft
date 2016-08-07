//! [Discrete Fourier transform][1].
//!
//! The `Transform` trait is responsible for performing the transform. The trait
//! is implemented for both real and complex data. There are three transform
//! operations available: forward, backward, and inverse. The desired operation
//! is specified by the `Operation` enumeration passed to the `Plan::new`
//! function, which precomputes auxiliary information needed for
//! `Transform::transform`. All the operations are preformed in place.
//!
//! When applied to real data, `Transform::transform` works as follows. If the
//! operation is `Operation::Forward`, the data are replaced by the positive
//! frequency half of their complex Fourier transform. The first and last
//! components of the complex transform, which are real, are stored in `self[0]`
//! and `self[1]`, respectively. Regarding the other two operations, the data
//! are assumed to be packed in the above format. See the reference below for
//! further details.
//!
//! ## Example
//!
//! ```
//! use dft::{Operation, Plan, c64};
//!
//! let plan = Plan::new(Operation::Forward, 512);
//! let mut data = vec![c64::new(42.0, 69.0); 512];
//! dft::transform(&mut data, &plan);
//! ```
//!
//! ## References
//!
//! 1. W. Press, S. Teukolsky, W. Vetterling, and B. Flannery, “Numerical
//! Recipes 3rd Edition: The Art of Scientific Computing,” Cambridge University
//! Press, 2007.
//!
//! [1]: https://en.wikipedia.org/wiki/Discrete_Fourier_transform

extern crate num_complex;
extern crate num_traits;

use num_complex::Complex;
use num_traits::Float;

/// A complex number with 32-bit parts.
#[allow(non_camel_case_types)]
pub type c32 = Complex<f32>;

/// A complex number with 64-bit parts.
#[allow(non_camel_case_types)]
pub type c64 = Complex<f64>;

macro_rules! c(($re:expr, $im:expr) => (::num_complex::Complex::new($re, $im)));

mod complex;
mod real;

pub use real::unpack;

/// A transform operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// The forward transform.
    Forward,
    /// The backward transform.
    Backward,
    /// The inverse transform.
    Inverse,
}

/// A transform plan.
#[derive(Clone, Debug)]
pub struct Plan<T> {
    n: usize,
    factors: Vec<Complex<T>>,
    operation: Operation,
}

/// The transform.
pub trait Transform<T> {
    /// Perform the transform.
    fn transform(&mut self, &Plan<T>);
}

impl<T> Plan<T> where T: Float {
    /// Create a plan for a specific operation and specific number of points.
    ///
    /// The number of points should be a power of two.
    pub fn new(operation: Operation, n: usize) -> Self {
        assert!(n.is_power_of_two());
        let zero = T::zero();
        let one = T::one();
        let two = one + one;
        let pi = T::acos(-one);
        let mut factors = vec![];
        let sign = if let Operation::Forward = operation { -one } else { one };
        let mut step = 1;
        while step < n {
            let (multiplier, mut factor) = {
                let theta = pi / T::from(step).unwrap();
                let sine = (theta / two).sin();
                (c!(-two * sine * sine, sign * theta.sin()), c!(one, zero))
            };
            for _ in 0..step {
                factors.push(factor);
                factor = multiplier * factor + factor;
            }
            step <<= 1;
        }
        Plan { n: n, factors: factors, operation: operation }
    }
}

/// Perform the transform.
///
/// The function is a shortcut for `Transform::transform`.
#[inline(always)]
pub fn transform<D: ?Sized, T>(data: &mut D, plan: &Plan<T>) where D: Transform<T> {
    Transform::transform(data, plan);
}
