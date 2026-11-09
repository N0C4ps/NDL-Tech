//! Visualizações de recursos: SRV, RTV, DSV, UAV.
//!
//! O D3D11 separa o recurso físico de suas visualizações. Uma mesma
//! textura pode ter múltiplas views, desde que flags, formato e regras de
//! compatibilidade permitam.
//!
//! ## Estado atual
//!
//! [`RenderTargetView`] (RTV), [`ShaderResourceView`] (SRV),
//! [`DepthStencilView`] (DSV) e [`UnorderedAccessView`] (UAV) já são
//! reais.
//!
//! Métodos de conveniência planejados para todas as views: `resource()`,
//! `descriptor()`, `format()`, `dimension()`, `mip_range()`,
//! `array_layer_range()`, `set_label()`. Para UAVs com contador:
//! `initial_count()`, `reset_counter()`, `preserve_counter()`,
//! `copy_structure_count()`.

use crate::backend::{ActiveBackend, Backend};

/// Uma visualização de alvo de renderização — o que permite a um contexto
/// desenhar/limpar dentro de uma textura (ex: o back buffer de uma
/// swapchain).
pub struct RenderTargetView {
    pub(crate) raw: <ActiveBackend as Backend>::RawRenderTargetView,
    pub(crate) format: crate::format::Format,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl RenderTargetView {
    pub(crate) fn from_raw(
        raw: <ActiveBackend as Backend>::RawRenderTargetView,
        format: crate::format::Format,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            raw,
            format,
            width,
            height,
        }
    }
}

impl RenderTargetView {
    pub fn format(&self) -> crate::format::Format {
        self.format
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Uma visualização de leitura por shader — o que permite amostrar uma
/// textura (ex: `Texture2D`/`Sampler` num pixel shader).
pub struct ShaderResourceView {
    pub(crate) raw: <ActiveBackend as Backend>::RawShaderResourceView,
}

impl ShaderResourceView {
    pub(crate) fn from_raw(raw: <ActiveBackend as Backend>::RawShaderResourceView) -> Self {
        Self { raw }
    }
}

/// Uma visualização de profundidade/stencil — o que permite a um
/// contexto testar/escrever profundidade durante o desenho.
pub struct DepthStencilView {
    pub(crate) raw: <ActiveBackend as Backend>::RawDepthStencilView,
    pub(crate) format: crate::format::Format,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl DepthStencilView {
    pub(crate) fn from_raw(
        raw: <ActiveBackend as Backend>::RawDepthStencilView,
        format: crate::format::Format,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            raw,
            format,
            width,
            height,
        }
    }
}

impl DepthStencilView {
    pub fn format(&self) -> crate::format::Format {
        self.format
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Uma visualização de leitura/escrita não ordenada — o que permite a um
/// compute shader ler e escrever um [`crate::buffer::Buffer`]
/// (`BufferUsage::Storage`).
pub struct UnorderedAccessView {
    pub(crate) raw: <ActiveBackend as Backend>::RawUnorderedAccessView,
}

impl UnorderedAccessView {
    pub(crate) fn from_raw(raw: <ActiveBackend as Backend>::RawUnorderedAccessView) -> Self {
        Self { raw }
    }
}
