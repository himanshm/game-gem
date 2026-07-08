//! Type-safe angle wrapper to prevent radian/degree confusion.

/// A type-safe angle that stores radians internally but provides
/// construction and conversion in both radians and degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Angle(f32);

impl Angle {
    /// Create from radians.
    #[inline]
    pub const fn from_radians(rad: f32) -> Self {
        Angle(rad)
    }

    /// Create from degrees.
    #[inline]
    pub fn from_degrees(deg: f32) -> Self {
        Angle(deg.to_radians())
    }

    /// Get the angle in radians.
    #[inline]
    pub fn as_radians(self) -> f32 {
        self.0
    }

    /// Get the angle in degrees.
    #[inline]
    pub fn as_degrees(self) -> f32 {
        self.0.to_degrees()
    }

    /// Zero angle.
    pub const fn zero() -> Self {
        Angle(0.0)
    }

    /// Normalize to the range (-π, π].
    pub fn normalized(self) -> Self {
        let mut a = self.0 % (2.0 * std::f32::consts::PI);
        if a > std::f32::consts::PI {
            a -= 2.0 * std::f32::consts::PI;
        } else if a < -std::f32::consts::PI {
            a += 2.0 * std::f32::consts::PI;
        }
        Angle(a)
    }

    /// Sine of the angle.
    #[inline]
    pub fn sin(self) -> f32 {
        self.0.sin()
    }

    /// Cosine of the angle.
    #[inline]
    pub fn cos(self) -> f32 {
        self.0.cos()
    }

    /// Both sin and cos, returned as a tuple (sin, cos).
    #[inline]
    pub fn sin_cos(self) -> (f32, f32) {
        self.0.sin_cos()
    }

    /// Tangent of the angle.
    #[inline]
    pub fn tan(self) -> f32 {
        self.0.tan()
    }
}

impl std::ops::Add for Angle {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Angle(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Angle(self.0 - rhs.0)
    }
}

impl std::ops::Mul<f32> for Angle {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Angle(self.0 * rhs)
    }
}

impl std::ops::Div<f32> for Angle {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Angle(self.0 / rhs)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Angle(-self.0)
    }
}

impl std::ops::AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degrees_to_radians() {
        let a = Angle::from_degrees(180.0);
        assert!((a.as_radians() - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let a = Angle::from_degrees(450.0).normalized();
        // Use a tolerance that accommodates f32 round-trip error from
        // degrees -> radians -> modulo -> degrees.
        assert!((a.as_degrees() - 90.0).abs() < 1e-4);
    }
}