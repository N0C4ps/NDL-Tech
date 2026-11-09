//! Teste e escrita de profundidade e stencil.
//!
//! Essencial para qualquer cena 3D com mais de um objeto: sem um depth
//! buffer e um `DepthStencilState` com teste ativado, a geometria é
//! desenhada na ordem em que os `draw()` acontecem, não na ordem real de
//! profundidade — objetos atrás aparecem por cima dos da frente.
//!
//! ## Estado atual
//!
//! [`DepthStencilState`] e [`CompareOp`] já são reais, cobrindo teste e
//! escrita de profundidade. Teste de stencil (`StencilOp`, reference,
//! read/write mask, front/back face) continua reservado.

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

/// Configuração usada para criar um [`DepthStencilState`].
#[derive(Debug, Clone, Copy)]
pub struct DepthStencilStateDescriptor {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareOp,
}

impl DepthStencilStateDescriptor {
    /// Sem teste nem escrita de profundidade — equivalente a não ter
    /// depth buffer nenhum.
    pub const fn disabled() -> Self {
        Self {
            depth_test_enabled: false,
            depth_write_enabled: false,
            depth_compare: CompareOp::Always,
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
        }
    }

    /// Testa profundidade mas não escreve — útil para passes de
    /// transparência, desenhados depois da geometria opaca.
    pub const fn depth_read_only() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: false,
            depth_compare: CompareOp::LessEqual,
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
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11DepthStencilState,
}

impl Device {
    pub fn create_depth_stencil_state(
        &self,
        desc: &DepthStencilStateDescriptor,
    ) -> Result<DepthStencilState, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_depth_stencil_state(self, desc);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = desc;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
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
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::create_depth_stencil_view(self, width, height, format);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (width, height, format);
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }
}
