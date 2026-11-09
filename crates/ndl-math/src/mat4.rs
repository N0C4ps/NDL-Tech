use crate::{Quat, Vec3, Vec4};
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

    /// Matriz de rotação equivalente ao quaternion `q` — o jeito normal de
    /// obter uma rotação sem gimbal lock ([`Quat::from_axis_angle`]) no
    /// formato que a GPU/`mul_vec4` esperam.
    pub fn from_quat(q: Quat) -> Self {
        let q = q.normalize();
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        let (xx, yy, zz) = (x * x, y * y, z * z);
        let (xy, xz, yz) = (x * y, x * z, y * z);
        let (wx, wy, wz) = (w * x, w * y, w * z);
        Mat4::from_cols(
            Vec4::new(1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0),
            Vec4::new(2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0),
            Vec4::new(2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    /// Matriz de projeção em perspectiva, convenção left-handed (a mesma
    /// usada por Direct3D): profundidade de clip space em `[0, 1]`, não
    /// `[-1, 1]` como OpenGL — compatível direto com o `MinDepth`/
    /// `MaxDepth` de um viewport D3D11 e com `DepthStencilState` sem
    /// nenhuma conversão extra. `fov_y_radians` é o campo de visão
    /// vertical; `aspect` é largura/altura.
    pub fn perspective_lh(fov_y_radians: f32, aspect: f32, z_near: f32, z_far: f32) -> Self {
        let f = 1.0 / (fov_y_radians / 2.0).tan();
        Mat4::from_cols(
            Vec4::new(f / aspect, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, z_far / (z_far - z_near), 1.0),
            Vec4::new(0.0, 0.0, -z_near * z_far / (z_far - z_near), 0.0),
        )
    }

    /// Matriz de câmera (view), convenção left-handed, olhando de `eye`
    /// para `target` com `up` definindo a orientação vertical.
    pub fn look_at_lh(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let z_axis = (target - eye).normalize();
        let x_axis = up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);
        Mat4::from_cols(
            Vec4::new(x_axis.x, y_axis.x, z_axis.x, 0.0),
            Vec4::new(x_axis.y, y_axis.y, z_axis.y, 0.0),
            Vec4::new(x_axis.z, y_axis.z, z_axis.z, 0.0),
            Vec4::new(-x_axis.dot(eye), -y_axis.dot(eye), -z_axis.dot(eye), 1.0),
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

    #[test]
    fn quat_rotation_matches_axis_angle() {
        use std::f32::consts::PI;
        // +X rotacionado 90° em torno de +Y (regra da mão direita) vai
        // para -Z.
        let q = Quat::from_axis_angle(Vec3::Y, PI / 2.0);
        let m = Mat4::from_quat(q);
        let rotated = m.mul_vec4(Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert!((rotated.x - 0.0).abs() < 1e-5);
        assert!((rotated.y - 0.0).abs() < 1e-5);
        assert!((rotated.z - -1.0).abs() < 1e-5);
    }

    #[test]
    fn quat_identity_is_identity_matrix() {
        assert_eq!(Mat4::from_quat(Quat::IDENTITY), Mat4::IDENTITY);
    }

    #[test]
    fn perspective_maps_near_and_far_plane_depth() {
        let proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 100.0);

        // No plano near, profundidade de clip space / w deve virar 0.
        let near = proj.mul_vec4(Vec4::new(0.0, 0.0, 1.0, 1.0));
        assert!((near.z / near.w - 0.0).abs() < 1e-5);

        // No plano far, deve virar 1 — o intervalo [0, 1] que o D3D11
        // espera (ver DepthStencilState/Viewport), não [-1, 1] como OpenGL.
        let far = proj.mul_vec4(Vec4::new(0.0, 0.0, 100.0, 1.0));
        assert!((far.z / far.w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn perspective_preserves_aspect_ratio_in_x() {
        let proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_2, 2.0, 1.0, 100.0);
        // Largura dobrada -> mesma coluna de X precisa comprimir pela
        // metade para o ponto continuar na borda do frustum.
        assert!((proj.cols[0].x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn look_at_puts_target_on_the_forward_axis() {
        let eye = Vec3::new(0.0, 0.0, -5.0);
        let target = Vec3::ZERO;
        let view = Mat4::look_at_lh(eye, target, Vec3::Y);

        // O alvo, visto da câmera, precisa estar à frente (z positivo) e
        // centralizado (x == y == 0) no espaço de view.
        let target_in_view = view.mul_vec4(Vec4::new(target.x, target.y, target.z, 1.0));
        assert!((target_in_view.x - 0.0).abs() < 1e-5);
        assert!((target_in_view.y - 0.0).abs() < 1e-5);
        assert!(target_in_view.z > 0.0);
    }

    #[test]
    fn look_at_keeps_eye_at_the_origin_of_view_space() {
        let eye = Vec3::new(3.0, 2.0, -5.0);
        let view = Mat4::look_at_lh(eye, Vec3::ZERO, Vec3::Y);
        let eye_in_view = view.mul_vec4(Vec4::new(eye.x, eye.y, eye.z, 1.0));
        assert!(eye_in_view.x.abs() < 1e-4);
        assert!(eye_in_view.y.abs() < 1e-4);
        assert!(eye_in_view.z.abs() < 1e-4);
    }
}
