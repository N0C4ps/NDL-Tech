/// Limites numéricos conhecidos para o dispositivo criado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLimits {
    /// Tamanho máximo de buffer, em bytes.
    pub max_buffer_size: u64,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_color_attachments: u32,
    pub max_sampler_anisotropy: u32,
    pub max_storage_buffer_bindings: u32,
    pub max_uniform_buffer_bindings: u32,
    pub max_sampled_resource_bindings: u32,
    pub max_sampler_bindings: u32,
    pub max_vertex_buffer_bindings: u32,
    pub max_viewport_dimension: u32,
    pub max_compute_workgroups_per_dimension: u32,
}

/// Origem de uma informação consultada ou publicada pelo backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum CapabilitySource {
    /// Consultada diretamente no dispositivo ou driver.
    DeviceQuery,
    /// Garantida pelo contrato normativo da API ativa.
    ApiGuarantee,
    /// O backend não conseguiu consultar o valor e usou uma suposição conservadora.
    #[default]
    ConservativeFallback,
}

/// Compatibilidade com o nome histórico usado para a origem dos limites.
pub type LimitsSource = CapabilitySource;

/// Suporte observado para um recurso opcional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FeatureSupport {
    pub supported: bool,
    pub source: CapabilitySource,
}

/// Recursos opcionais que o dispositivo gráfico atual suporta.
///
/// Cada recurso carrega a origem de sua informação. Assim, uma capacidade
/// consultada no driver não se confunde com outra inferida de uma garantia da
/// API ou tratada como indisponível por falta de consulta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeviceCapabilities {
    pub compute: FeatureSupport,
    pub tessellation: FeatureSupport,
    pub logic_ops: FeatureSupport,
}

impl DeviceCapabilities {
    pub fn supports_compute(self) -> bool {
        self.compute.supported
    }

    pub fn supports_tessellation(self) -> bool {
        self.tessellation.supported
    }

    pub fn supports_logic_ops(self) -> bool {
        self.logic_ops.supported
    }
}

/// Capacidades consultadas para um formato no dispositivo ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatFeatures {
    pub texture_2d: bool,
    pub sampled: bool,
    pub render_target: bool,
    pub depth_stencil: bool,
    pub vertex_buffer: bool,
    pub storage: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_capability_record_defaults_to_unsupported() {
        let capabilities = DeviceCapabilities::default();
        assert!(!capabilities.supports_compute());
        assert!(!capabilities.supports_tessellation());
        assert!(!capabilities.supports_logic_ops());
    }

    #[test]
    fn limits_source_describes_api_neutral_provenance() {
        assert_ne!(LimitsSource::ApiGuarantee, LimitsSource::DeviceQuery);
        assert_ne!(
            LimitsSource::ConservativeFallback,
            LimitsSource::DeviceQuery
        );
    }
}
