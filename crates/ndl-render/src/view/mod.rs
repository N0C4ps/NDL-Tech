//! Visualizações de recursos: SRV, RTV, DSV, UAV.
//!
//! O D3D11 separa o recurso físico de suas visualizações. Uma mesma
//! textura pode ter múltiplas views, desde que flags, formato e regras de
//! compatibilidade permitam.
//!
//! ## Estado atual
//!
//! [`RenderTargetView`] (RTV) e [`ShaderResourceView`] (SRV) já são
//! reais. `DepthStencilView` e `UnorderedAccessView` ainda não existem;
//! entram quando `depth_stencil`/`compute` forem implementados de verdade.
//!
//! Métodos de conveniência planejados para todas as views: `resource()`,
//! `descriptor()`, `format()`, `dimension()`, `mip_range()`,
//! `array_layer_range()`, `set_label()`. Para UAVs com contador:
//! `initial_count()`, `reset_counter()`, `preserve_counter()`,
//! `copy_structure_count()`.

/// Uma visualização de alvo de renderização — o que permite a um contexto
/// desenhar/limpar dentro de uma textura (ex: o back buffer de uma
/// swapchain).
pub struct RenderTargetView {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11RenderTargetView,
}

impl RenderTargetView {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(raw: windows::Win32::Graphics::Direct3D11::ID3D11RenderTargetView) -> Self {
        Self { raw }
    }
}

/// Uma visualização de leitura por shader — o que permite amostrar uma
/// textura (ex: `Texture2D`/`Sampler` num pixel shader).
pub struct ShaderResourceView {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11ShaderResourceView,
}

impl ShaderResourceView {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11ShaderResourceView,
    ) -> Self {
        Self { raw }
    }
}
