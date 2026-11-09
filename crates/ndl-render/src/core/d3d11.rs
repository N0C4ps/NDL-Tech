//! Extensoes nativas para integrar diretamente com Direct3D 11.
//!
//! Esta API e especifica do Windows/D3D11 e fica separada da superficie
//! portavel [`super::Device`]. Chamadas externas que alterem estado ou
//! lifetime do device/contexto podem invalidar invariantes internos do
//! renderer; o chamador precisa sincronizar esses acessos com `ndl-render`.

use super::Device;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11DeviceContext};

/// Retorna o device nativo D3D11 pertencente a `device`.
///
/// # Safety
/// O chamador nao deve liberar nem substituir o device, e deve manter a
/// sincronizacao com quaisquer operacoes de `ndl-render`.
pub unsafe fn device(device: &Device) -> &ID3D11Device {
    &device.raw.device
}

/// Retorna o contexto imediato D3D11 compartilhado com `ndl-render`.
///
/// # Safety
/// O contexto possui estado mutavel compartilhado com comandos de
/// `ndl-render`. O chamador deve sincronizar todos os acessos e nao deve
/// alterar estado que invalide operacoes subsequentes do renderer.
pub unsafe fn immediate_context(device: &Device) -> &ID3D11DeviceContext {
    &device.raw.context
}
