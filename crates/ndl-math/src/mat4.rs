use crate::Vec4;
use std::ops::Mul;

/// Matriz 4x4 column-major, no padrão usado por APIs gráficas (wgpu/DirectX/Vulkan).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Colunas da matriz.
    pub cols: [Vec4; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        cols: [
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ],
    };

    pub const fn from_cols(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Self {
            cols: [c0, c1, c2, c3],
        }
    }

    pub fn from_translation(t: crate::Vec3) -> Self {
        let mut m = Mat4::IDENTITY;
        m.cols[3] = Vec4::new(t.x, t.y, t.z, 1.0);
        m
    }

    pub fn from_scale(s: crate::Vec3) -> Self {
        Mat4::from_cols(
            Vec4::new(s.x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, s.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, s.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn transpose(self) -> Mat4 {
        let c = self.cols;
        Mat4::from_cols(
            Vec4::new(c[0].x, c[1].x, c[2].x, c[3].x),
            Vec4::new(c[0].y, c[1].y, c[2].y, c[3].y),
            Vec4::new(c[0].z, c[1].z, c[2].z, c[3].z),
            Vec4::new(c[0].w, c[1].w, c[2].w, c[3].w),
        )
    }

    pub fn mul_vec4(self, v: Vec4) -> Vec4 {
        self.cols[0] * v.x + self.cols[1] * v.y + self.cols[2] * v.z + self.cols[3] * v.w
    }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: Mat4) -> Mat4 {
        Mat4::from_cols(
            self.mul_vec4(rhs.cols[0]),
            self.mul_vec4(rhs.cols[1]),
            self.mul_vec4(rhs.cols[2]),
            self.mul_vec4(rhs.cols[3]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec3;

    #[test]
    fn identity_mul() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(m * Mat4::IDENTITY, m);
        assert_eq!(Mat4::IDENTITY * m, m);
    }

    #[test]
    fn translation_applies_to_point() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let p = Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert_eq!(m.mul_vec4(p), Vec4::new(1.0, 2.0, 3.0, 1.0));
    }

    #[test]
    fn scale_applies_to_point() {
        let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let p = Vec4::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(m.mul_vec4(p), Vec4::new(2.0, 3.0, 4.0, 1.0));
    }
}
