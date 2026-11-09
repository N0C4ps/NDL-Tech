//! Teste e escrita de profundidade e stencil.
//!
//! Essencial para qualquer cena 3D com mais de um objeto: sem um depth
//! buffer e um `DepthStencilState` com teste ativado, a geometria é
//! desenhada na ordem em que os `draw()` acontecem, não na ordem real de
//! profundidade — objetos atrás aparecem por cima dos da frente.
//!
//! ## Estado atual
//!
//! [`DepthStencilState`] cobre teste/escrita de profundidade e stencil,
//! incluindo operações por face, compare e máscaras read/write. A
//! referência de stencil é dinâmica e atualmente usa zero ao vincular o
//! pipeline; setters dinâmicos por draw ainda estão pendentes.

use crate::backend::{ActiveBackend, Backend};
use crate::core::{Device, GraphicsError};

/// Função de comparação usada no teste de profundidade/stencil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    #[default]
    Always,
}

/// Operação executada em um valor de stencil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StencilOp {
    #[default]
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

/// Regras de stencil aplicadas a uma orientação de face.
#[derive(Debug, Clone, Copy)]
pub struct StencilFaceDescriptor {
    pub fail_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub compare: CompareOp,
}

impl Default for StencilFaceDescriptor {
    fn default() -> Self {
        Self::keep()
    }
}

impl StencilFaceDescriptor {
    pub const fn keep() -> Self {
        Self {
            fail_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            compare: CompareOp::Always,
        }
    }
}

/// Configuração usada para criar um [`DepthStencilState`].
#[derive(Debug, Clone, Copy)]
pub struct DepthStencilStateDescriptor {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareOp,
    pub stencil_test_enabled: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub front_face: StencilFaceDescriptor,
    pub back_face: StencilFaceDescriptor,
}

impl DepthStencilStateDescriptor {
    /// Sem teste nem escrita de profundidade — equivalente a não ter
    /// depth buffer nenhum.
    pub const fn disabled() -> Self {
        Self {
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: CompareOp::Always,
            stencil_test_enabled: false,
            stencil_read_mask: 0xff,
            stencil_write_mask: 0xff,
            front_face: StencilFaceDescriptor::keep(),
            back_face: StencilFaceDescriptor::keep(),
        }
    }

    /// O caso comum pra 3D: testa e escreve profundidade, desenhando por
    /// cima quando o novo fragmento está mais perto da câmera (profundidade
    /// convencional, 0 = perto, 1 = longe).
    pub const fn depth_read_write() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: true,
            depth_compare: CompareOp::LessEqual,
            stencil_test_enabled: false,
            stencil_read_mask: 0xff,
            stencil_write_mask: 0xff,
            front_face: StencilFaceDescriptor::keep(),
            back_face: StencilFaceDescriptor::keep(),
        }
    }

    /// Testa profundidade mas não escreve — útil para passes de
    /// transparência, desenhados depois da geometria opaca.
    pub const fn depth_read_only() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: false,
            depth_compare: CompareOp::LessEqual,
            stencil_test_enabled: false,
            stencil_read_mask: 0xff,
            stencil_write_mask: 0xff,
            front_face: StencilFaceDescriptor::keep(),
            back_face: StencilFaceDescriptor::keep(),
        }
    }
}

impl Default for DepthStencilStateDescriptor {
    fn default() -> Self {
        Self::depth_read_write()
    }
}

/// Um estado de profundidade/stencil já criado no dispositivo.
pub struct DepthStencilState {
    pub(crate) raw: <ActiveBackend as Backend>::RawDepthStencilState,
}

impl Device {
    pub fn create_depth_stencil_state(
        &self,
        desc: &DepthStencilStateDescriptor,
    ) -> Result<DepthStencilState, GraphicsError> {
        ActiveBackend::create_depth_stencil_state(self, desc)
    }

    /// Cria uma textura de profundidade (sem dados iniciais — profundidade
    /// é escrita pela GPU durante o desenho, não enviada pela CPU) e sua
    /// [`crate::view::DepthStencilView`] correspondente.
    pub fn create_depth_texture(
        &self,
        width: u32,
        height: u32,
        format: crate::format::Format,
    ) -> Result<crate::view::DepthStencilView, GraphicsError> {
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidArgument {
                name: "depth texture extent",
                reason: "largura e altura precisam ser maiores que zero".into(),
            });
        }
        let limit = self.limits().max_texture_dimension_2d;
        if width > limit || height > limit {
            return Err(GraphicsError::InvalidArgument {
                name: "depth texture extent",
                reason: format!(
                    "{}x{} excede o limite 2D {}x{}",
                    width, height, limit, limit
                ),
            });
        }
        if !format.is_depth() {
            return Err(GraphicsError::IncompatibleResource {
                operation: "create_depth_texture",
                reason: format!("{format:?} não é um formato depth/stencil"),
            });
        }
        if !self.format_features(format)?.depth_stencil {
            return Err(GraphicsError::FeatureNotSupported {
                feature: format!("depth/stencil no formato {format:?}"),
            });
        }
        ActiveBackend::create_depth_stencil_view(self, width, height, format)
    }
}
