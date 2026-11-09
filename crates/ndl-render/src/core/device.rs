use super::capabilities::{DeviceCapabilities, DeviceLimits};
use super::error::GraphicsError;

/// Configuração usada para criar um [`Device`].
#[derive(Debug, Clone, Default)]
pub struct DeviceDescriptor {
    pub label: Option<String>,
    /// Pede o adaptador de software (WARP, no Direct3D) em vez de uma GPU
    /// de hardware — útil em CI sem GPU dedicada.
    pub prefer_software: bool,
    /// Ativa a debug layer do backend, quando disponível (ver `crate::debug`).
    pub enable_debug_layer: bool,
}

/// Um dispositivo gráfico inicializado — o ponto de partida para criar
/// todo o resto (buffers, texturas, pipelines, ...).
pub struct Device {
    capabilities: DeviceCapabilities,
    #[cfg(target_os = "windows")]
    pub(crate) raw: crate::backend::directx::RawDevice,
}

impl Device {
    /// Cria o dispositivo padrão (hardware, a menos que
    /// `descriptor.prefer_software` seja `true`).
    pub fn new(descriptor: &DeviceDescriptor) -> Result<Self, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            let (raw, is_software) = crate::backend::directx::create_default_device(descriptor)?;
            let feature_level = crate::backend::directx::map_feature_level(raw.feature_level);
            let capabilities = DeviceCapabilities::from_feature_level(feature_level, is_software);
            return Ok(Self { capabilities, raw });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = descriptor;
            Err(GraphicsError::Unsupported(
                "nenhum backend gráfico implementado para este SO ainda",
            ))
        }
    }

    pub fn capabilities(&self) -> DeviceCapabilities {
        self.capabilities
    }

    pub fn limits(&self) -> DeviceLimits {
        self.capabilities.feature_level.default_limits()
    }

    pub fn is_software(&self) -> bool {
        self.capabilities.is_software
    }

    /// O contexto imediato deste dispositivo — por onde comandos de
    /// limpeza/desenho são submetidos.
    pub fn context(&self) -> crate::context::Context {
        #[cfg(target_os = "windows")]
        {
            crate::context::Context::from_raw(self.raw.context.clone())
        }
        #[cfg(not(target_os = "windows"))]
        {
            crate::context::Context {}
        }
    }

    /// Se o dispositivo foi perdido (driver crashou, GPU removida, etc.) e
    /// precisa ser recriado.
    pub fn is_lost(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            crate::backend::directx::is_device_lost(&self.raw)
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}
