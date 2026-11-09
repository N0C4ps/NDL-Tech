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

/// Uma visualização de profundidade/stencil — o que permite a um
/// contexto testar/escrever profundidade durante o desenho.
pub struct DepthStencilView {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11DepthStencilView,
}

impl DepthStencilView {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11DepthStencilView,
    ) -> Self {
        Self { raw }
    }
}

/// Uma visualização de leitura/escrita não ordenada — o que permite a um
/// compute shader ler e escrever um [`crate::buffer::Buffer`]
/// (`BufferUsage::Storage`).
pub struct UnorderedAccessView {
    #[cfg(target_os = "windows")]
    pub(crate) raw: windows::Win32::Graphics::Direct3D11::ID3D11UnorderedAccessView,
}

impl UnorderedAccessView {
    #[cfg(target_os = "windows")]
    pub(crate) fn from_raw(
        raw: windows::Win32::Graphics::Direct3D11::ID3D11UnorderedAccessView,
    ) -> Self {
        Self { raw }
    }
}
