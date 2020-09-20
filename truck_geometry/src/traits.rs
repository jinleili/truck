use crate::*;
use cgmath::AbsDiffEq;
use std::ops::*;

/// Defines a tolerance in the whole package
pub trait Tolerance: AbsDiffEq<Epsilon = f64> + Debug {
    /// The "distance" is less than `TOLERANCE`.
    fn near(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE) }

    /// The "distance" is less than `TOLERANCR2`.
    fn near2(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE2) }

    /// assert if `one` is not near `other`.
    fn assert_near(one: &Self, other: &Self) {
        cgmath::assert_abs_diff_eq!(one, other, epsilon = TOLERANCE)
    }

    /// assertion if `one` is not near `other` in square order.
    fn assert_near2(one: &Self, other: &Self) {
        cgmath::assert_abs_diff_eq!(one, other, epsilon = TOLERANCE2)
    }
}

/// The structs defined the origin. `f64`, `Vector`, and so on.
pub trait Origin: Tolerance + Zero {
    /// near origin
    #[inline(always)]
    fn so_small(&self) -> bool { self.near(&Self::zero()) }

    /// near origin in square order
    #[inline(always)]
    fn so_small2(&self) -> bool { self.near2(&Self::zero()) }
}

/// Rounds the value by tolerance
/// # Example
/// ```
/// use truck_geometry::*;
/// assert_eq!(1.23456789_f64.round_by_tolerance(), &1.2345678);
/// ```
pub trait RoundByTolerance: Tolerance {
    /// Rounds the value by tolerance
    fn round_by_tolerance(&mut self) -> &mut Self;
}

impl Tolerance for f64 {}
impl Tolerance for [f64] {}
impl Origin for f64 {}

impl RoundByTolerance for f64 {
    fn round_by_tolerance(&mut self) -> &mut f64 {
        *self = (*self / TOLERANCE).floor() * TOLERANCE;
        self
    }
}

macro_rules! impl_tolerance {
    ($typename: ty, $($num: expr),+) => {
        impl Tolerance for $typename {}
        impl Origin for $typename {}
        impl RoundByTolerance for $typename {
            #[inline(always)]
            fn round_by_tolerance(&mut self) -> &mut Self {
                $(self[$num].round_by_tolerance();)+
                self
            }
        }
    };
}

impl_tolerance!(Vector1, 0);
impl_tolerance!(Vector2, 0, 1);
impl_tolerance!(Vector3, 0, 1, 2);
impl_tolerance!(Vector4, 0, 1, 2, 3);
impl_tolerance!(Matrix2, 0, 1, 2, 3);
impl_tolerance!(Matrix3, 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_tolerance!(Matrix4, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

impl Tolerance for Point1 {}
impl Tolerance for Point2 {}
impl Tolerance for Point3 {}

/// The projection to the plane whose the last component is `1.0`.
/// In other words, the transform to the homogeneous coordinate of
/// the (n-1)-dim affine space.
pub trait RationalProjective: InnerSpace<Scalar = f64> + Origin {
    /// The (n-1)-dim vector space
    type Rationalized: InnerSpace<Scalar = f64> + Origin;
    #[doc(hidden)]
    fn truncate(&self) -> Self::Rationalized;
    #[doc(hidden)]
    fn last(&self) -> f64;

    /// Returns the projection to the plane whose the last component is `1.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = Vector4::new(8.0, 4.0, 6.0, 2.0).rational_projection();
    /// assert_eq!(v, Vector3::new(4.0, 2.0, 3.0));
    /// ```
    #[inline(always)]
    fn rational_projection(&self) -> Self::Rationalized { self.truncate() / self.last() }
    
    /// Returns the derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = Vector4::new(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// let ans = Vector3::new(1.0, 2.0 * t, 3.0 * t * t);
    /// assert_eq!(pt.rational_derivation(der), ans);
    /// ```
    #[inline(always)]
    fn rational_derivation(&self, der: Self) -> Self::Rationalized {
        (der.truncate() * self.last() - self.truncate() * der.last()) / (self.last() * self.last())
    }
    
    /// Returns the secondary derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the 2nd ordered derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// * `der2` - the 2nd ordered derivation c''(t) of the curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = Vector4::new(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = Vector4::new(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the 2nd ord. deri.: c''(t) = (2, 6t, 12t^2, 0)
    /// let der2 = Vector4::new(2.0, 6.0 * t, 12.0 * t * t, 0.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// // the 2nd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 2, 6t, 0)
    /// let ans = Vector3::new(0.0, 2.0, 6.0 * t);
    /// assert_eq!(pt.rational_derivation2(der, der2), ans);
    /// ```
    #[inline(always)]
    fn rational_derivation2(&self, der: Self, der2: Self) -> Self::Rationalized {
        let pre_coef1 = der.last() / (self.last() * self.last());
        let coef1 = pre_coef1 + pre_coef1;
        let der_last2 = der.last() * der.last();
        let coef2 = (der_last2 + der_last2 - der2.last() * self.last())
            / (self.last() * self.last() * self.last());
        let res = der2 / self.last() - der * coef1 + *self * coef2;
        res.truncate()
    }

    /// Returns the cross derivation of the rational surface.
    ///
    /// For a surface s(u, v) = (s_0(u, v), s_1(u, v), s_2(u, v), s_3(u, v)), returns the derivation
    /// of the projected surface (s_0 / s_3, s_1 / s_3, s_2 / s_3) by both u and v.
    /// # Arguments
    /// * `self` - the point of the surface s(u, v)
    /// * `uder` - the u-derivation s_u(u, v) of the surface
    /// * `vder` - the v-derivation s_v(u, v) of the surface
    /// * `uvder` - the 2nd ordered derivation s_{uv}(u, v) of the surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// // calculate the derivation at (u, v) = (1.0, 2.0).
    /// let (u, v) = (1.0, 2.0);
    /// // the curve: s(u, v) = (u^3 v^2, u^2 v^3, u v, u)
    /// let pt = Vector4::new(
    ///     u * u * u * v * v,
    ///     u * u * v * v * v,
    ///     u * v,
    ///     u,
    /// );
    /// // the u-derivation: s_u(u, v) = (3u^2 v^2, 2u * v^3, v, 1)
    /// let uder = Vector4::new(
    ///     3.0 * u * u * v * v,
    ///     2.0 * u * v * v * v,
    ///     v,
    ///     1.0,
    /// );
    /// // the v-derivation: s_v(u, v) = (2u^3 v, 3u^2 v^2, u, 0)
    /// let vder = Vector4::new(
    ///     2.0 * u * u * u * v,
    ///     3.0 * u * u * v * v,
    ///     u,
    ///     0.0,
    /// );
    /// // s_{uv}(u, v) = (6u^2 v, 6u v^2, 1, 0)
    /// let uvder = Vector4::new(6.0 * u * u * v, 6.0 * u * v * v, 1.0, 0.0);
    /// // the projected surface: \bar{s}(u, v) = (u^2 v^2, u v^3, v)
    /// // \bar{s}_u(u, v) = (2u v^2, v^3, 0)
    /// // \bar{s}_v(u, v) = (2u^2 v, 3u v^2, 1)
    /// // \bar{s}_{uv}(u, v) = (4uv, 3v^2, 0)
    /// let ans = Vector3::new(4.0 * u * v, 3.0 * v * v, 0.0);
    /// assert_eq!(pt.rational_cross_derivation(uder, vder, uvder), ans);
    /// ```
    #[inline(always)]
    fn rational_cross_derivation(&self, uder: Self, vder: Self, uvder: Self) -> Self::Rationalized {
        let self_last2 = self.last() * self.last();
        let coef1 = vder.last() / self_last2;
        let coef2 = uder.last() / self_last2;
        let der_last2 = uder.last() * vder.last();
        let coef3 = (der_last2 + der_last2 - uvder.last() * self.last())
            / (self_last2 * self.last());
        let res = uvder / self.last() - uder * coef1 - vder * coef2 + *self * coef3;
        res.truncate()
    }
}

macro_rules! impl_rational {
    ($typename: ty, $rationalized: ty, $last: expr, $($num: expr),*) => {
        impl RationalProjective for $typename {
            type Rationalized = $rationalized;
            fn truncate(&self) -> $rationalized { <$rationalized>::new($(self[$num]),*) }
            fn last(&self) -> Self::Scalar { self[$last] }
        }
    };
}

impl_rational!(Vector2, Vector1, 1, 0);
impl_rational!(Vector3, Vector2, 2, 0, 1);
impl_rational!(Vector4, Vector3, 3, 0, 1, 2);

/// The trait for defining the bounding box
pub trait Bounded<S> {
    /// the result of subtraction
    type Vector;
    #[doc(hidden)]
    fn infinity() -> Self;
    #[doc(hidden)]
    fn neg_infinity() -> Self;
    #[doc(hidden)]
    fn max(&self, other: &Self) -> Self;
    #[doc(hidden)]
    fn min(&self, other: &Self) -> Self;
    #[doc(hidden)]
    fn max_component(one: Self::Vector) -> S;
    #[doc(hidden)]
    fn diagonal(self, other: Self) -> Self::Vector;
    #[doc(hidden)]
    fn mid(self, other: Self) -> Self;
}

mod impl_bounded {
    use cgmath::{Vector1, Vector2, Vector3, Vector4, Point1, Point2, Point3};
    macro_rules! pr2 {
        ($a: expr, $b: expr) => {
            $b
        };
    }
    macro_rules! impl_bounded {
        ($typename: ident, $vectortype: ident, $($num: expr),*) => {
            impl<S: cgmath::BaseFloat> super::Bounded<S> for $typename<S> {
                type Vector = $vectortype<S>;
                fn infinity() -> $typename<S> {
                    $typename::new($(pr2!($num, S::infinity())),*)
                }
                fn neg_infinity() -> $typename<S> {
                    $typename::new($(pr2!($num, S::neg_infinity())),*)
                }
                fn max(&self, other: &Self) -> Self {
                    $typename::new(
                        $(
                            if self[$num] < other[$num] {
                                other[$num]
                            } else {
                                self[$num]
                            }
                        ),*
                    )
                }
                fn min(&self, other: &Self) -> Self {
                    $typename::new(
                        $(
                            if self[$num] > other[$num] {
                                other[$num]
                            } else {
                                self[$num]
                            }
                        ),*
                    )
                }
                fn max_component(one: Self::Vector) -> S {
                    let mut max = S::neg_infinity();
                    $(if max < one[$num] { max = one[$num] })*
                    max
                }
                fn diagonal(self, other: Self) -> Self::Vector { self - other }
                fn mid(self, other: Self) -> Self {
                    self + (other - self) / (S::one() + S::one())
                }
            }
        };
    }
    impl_bounded!(Vector1, Vector1, 0);
    impl_bounded!(Point1, Vector1, 0);
    impl_bounded!(Vector2, Vector2, 0, 1);
    impl_bounded!(Point2, Vector2, 0, 1);
    impl_bounded!(Vector3, Vector3, 0, 1, 2);
    impl_bounded!(Point3, Vector3, 0, 1, 2);
    impl_bounded!(Vector4, Vector4, 0, 1, 2, 3);
}

/// The greedy trait for treating B-splines.
pub trait ExVectorSpace:
    RationalProjective
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<f64>
    + DivAssign<f64>
    + Index<usize, Output = f64>
    + Bounded<f64> {
}

impl ExVectorSpace for Vector2 {}
impl ExVectorSpace for Vector3 {}
impl ExVectorSpace for Vector4 {}

#[doc(hidden)]
#[inline(always)]
pub fn inv_or_zero(delta: f64) -> f64 {
    if delta.so_small() {
        0.0
    } else {
        1.0 / delta
    }
}