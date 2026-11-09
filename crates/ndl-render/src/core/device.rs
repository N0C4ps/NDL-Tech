use super::adapter::AdapterInfo;
use super::capabilities::{DeviceCapabilities, DeviceLimits, FormatFeatures, LimitsSource};
use super::error::GraphicsError;

/// Configuração usada para criar um [`Device`].
#[derive(Debug, Clone, Default)]
pub struct DeviceDescriptor {
    pub label: Option<String>,
    /// Prefere um adaptador de software, quando o backend oferece um,
    /// em vez de uma GPU de hardware - útil em CI sem GPU dedicada.
    pub prefer_software: bool,
    /// Ativa a debug layer do backend, quando disponível (ver `crate::debug`).
    pub enable_debug_layer: bool,
}

/// Um dispositivo gráfico inicializado — o ponto de partida para criar
/// todo o resto (buffers, texturas, pipelines, ...).
pub struct Device {
    capabilities: DeviceCapabilities,
    limits: DeviceLimits,
    limits_source: LimitsSource,
    adapter_info: AdapterInfo,
    bound_index_buffer: std::sync::Arc<std::sync::Mutex<Option<(u64, crate::input::IndexFormat)>>>,
    bound_vertex_buffers: crate::context::VertexBufferBindings,
    context_lock: crate::context::ContextLock,
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
            let (capabilities, limits, limits_source) =
                crate::backend::directx::device_properties(&raw);
            let adapter_info = crate::backend::directx::adapter_info_for_device(&raw)
                .unwrap_or_else(|_| AdapterInfo {
                    name: "Unknown graphics adapter".into(),
                    vendor_id: 0,
                    device_id: 0,
                    dedicated_video_memory: 0,
                    shared_system_memory: 0,
                    is_software,
                    device_type: if is_software {
                        super::AdapterType::Software
                    } else {
                        super::AdapterType::Unknown
                    },
                    driver_version: None,
                });
            let bound_index_buffer = std::sync::Arc::new(std::sync::Mutex::new(None));
            let bound_vertex_buffers =
                std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
            let context_lock = std::sync::Arc::new(std::sync::Mutex::new(()));
            return Ok(Self {
                capabilities,
                limits,
                limits_source,
                adapter_info,
                bound_index_buffer,
                bound_vertex_buffers,
                context_lock,
                raw,
            });
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
        self.limits
    }

    pub fn limits_source(&self) -> LimitsSource {
        self.limits_source
    }

    pub fn adapter_info(&self) -> &AdapterInfo {
        &self.adapter_info
    }

    pub fn backend_api(&self) -> super::BackendApi {
        #[cfg(target_os = "windows")]
        {
            super::BackendApi::Direct3D11
        }
        #[cfg(not(target_os = "windows"))]
        {
            super::BackendApi::Unknown
        }
    }

    /// Consulta o driver para saber quais usos são suportados por um formato.
    pub fn format_features(
        &self,
        format: crate::format::Format,
    ) -> Result<FormatFeatures, GraphicsError> {
        #[cfg(target_os = "windows")]
        {
            return crate::backend::directx::query_format_features(self, format);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = format;
            Err(GraphicsError::FeatureNotSupported {
                feature: "consulta de formatos sem backend ativo".into(),
            })
        }
    }

    pub fn is_software(&self) -> bool {
        self.adapter_info.is_software
    }

    /// O contexto imediato deste dispositivo — por onde comandos de
    /// limpeza/desenho são submetidos.
    pub fn context(&self) -> crate::context::Context {
        #[cfg(target_os = "windows")]
        {
            crate::context::Context::from_raw_with_shared(
                self.raw.context.clone(),
                self.bound_index_buffer.clone(),
                self.bound_vertex_buffers.clone(),
                self.context_lock.clone(),
                self.limits,
            )
        }
        #[cfg(not(target_os = "windows"))]
        {
            crate::context::Context {
                bound_index_buffer: self.bound_index_buffer.clone(),
                bound_vertex_buffers: self.bound_vertex_buffers.clone(),
                context_lock: self.context_lock.clone(),
                limits: self.limits,
            }
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
