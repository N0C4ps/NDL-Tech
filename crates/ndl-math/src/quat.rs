use crate::Vec3;
use std::ops::Mul;

/// Quaternion usado para representar rotações 3D sem gimbal lock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vec3, angle_rad: f32) -> Self {
        let axis = axis.normalize();
        let half = angle_rad * 0.5;
        let s = half.sin();
        Quat::new(axis.x * s, axis.y * s, axis.z * s, half.cos())
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Quat {
        let len = self.length();
        if len == 0.0 {
            Quat::IDENTITY
        } else {
            Quat::new(self.x / len, self.y / len, self.z / len, self.w / len)
        }
    }

    pub fn conjugate(self) -> Quat {
        Quat::new(-self.x, -self.y, -self.z, self.w)
    }
}

impl Mul for Quat {
    type Output = Quat;

    /// Composição de rotações: `self * rhs` aplica `rhs` primeiro, depois `self`.
    fn mul(self, rhs: Quat) -> Quat {
        Quat::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn identity_mul() {
        let q = Quat::from_axis_angle(Vec3::Y, PI / 2.0);
        assert_eq!(q * Quat::IDENTITY, q);
        assert_eq!(Quat::IDENTITY * q, q);
    }

    #[test]
    fn normalize() {
        let q = Quat::new(1.0, 2.0, 3.0, 4.0).normalize();
        assert!((q.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn conjugate_of_unit_is_inverse() {
        let q = Quat::from_axis_angle(Vec3::X, 1.3).normalize();
        let result = q * q.conjugate();
        assert!((result.w - 1.0).abs() < 1e-5);
        assert!(result.x.abs() < 1e-5);
        assert!(result.y.abs() < 1e-5);
        assert!(result.z.abs() < 1e-5);
    }
}
