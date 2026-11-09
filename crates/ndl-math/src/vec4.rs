use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
    pub const ONE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn dot(self, rhs: Vec4) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Vec4 {
        let len = self.length();
        if len == 0.0 {
            Vec4::ZERO
        } else {
            self / len
        }
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl Neg for Vec4 {
    type Output = Vec4;
    fn neg(self) -> Vec4 {
        Vec4::new(-self.x, -self.y, -self.z, -self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4::new(4.0, 3.0, 2.0, 1.0);
        assert_eq!(a + b, Vec4::new(5.0, 5.0, 5.0, 5.0));
        assert_eq!(a - b, Vec4::new(-3.0, -1.0, 1.0, 3.0));
    }

    #[test]
    fn dot_and_length() {
        let a = Vec4::new(1.0, 2.0, 2.0, 4.0);
        assert_eq!(a.length(), 5.0);
    }
}
