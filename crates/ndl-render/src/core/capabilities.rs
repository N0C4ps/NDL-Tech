/// Nível de recurso ("feature level") suportado pelo hardware/driver —
/// conceito do Direct3D, usado aqui para derivar limites conservadores
/// quando o backend não expõe algo mais preciso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureLevel {
    Level9_1,
    Level9_2,
    Level9_3,
    Level10_0,
    Level10_1,
    Level11_0,
    Level11_1,
}

impl FeatureLevel {
    /// Limites conhecidos/conservadores para este nível de recurso.
    ///
    /// Valores aproximados da especificação do Direct3D; onde o hardware
    /// real suporta mais, isso fica disponível só quando o backend
    /// consultar o driver diretamente (não é o caso ainda).
    pub fn default_limits(self) -> DeviceLimits {
        use FeatureLevel::*;
        match self {
            Level11_1 | Level11_0 => DeviceLimits {
                max_texture_dimension_2d: 16384,
                max_texture_dimension_3d: 2048,
                max_render_targets: 8,
                max_anisotropy: 16,
                max_uav_slots: 8,
                max_constant_buffer_slots: 14,
            },
            Level10_1 | Level10_0 => DeviceLimits {
                max_texture_dimension_2d: 8192,
                max_texture_dimension_3d: 2048,
                max_render_targets: 8,
                max_anisotropy: 16,
                max_uav_slots: 0,
                max_constant_buffer_slots: 14,
            },
            Level9_3 => DeviceLimits {
                max_texture_dimension_2d: 4096,
                max_texture_dimension_3d: 256,
                max_render_targets: 4,
                max_anisotropy: 16,
                max_uav_slots: 0,
                max_constant_buffer_slots: 14,
            },
            Level9_1 | Level9_2 => DeviceLimits {
                max_texture_dimension_2d: 2048,
                max_texture_dimension_3d: 256,
                max_render_targets: 1,
                max_anisotropy: 2,
                max_uav_slots: 0,
                max_constant_buffer_slots: 14,
            },
        }
    }

    pub fn supports_compute(self) -> bool {
        self >= FeatureLevel::Level10_0
    }

    pub fn supports_tessellation(self) -> bool {
        self >= FeatureLevel::Level11_0
    }

    pub fn supports_logic_ops(self) -> bool {
        self >= FeatureLevel::Level11_1
    }
}

/// Limites numéricos do dispositivo, derivados do [`FeatureLevel`] (por
/// enquanto — futuramente podem vir de uma consulta direta ao driver).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLimits {
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_render_targets: u32,
    pub max_anisotropy: u32,
    pub max_uav_slots: u32,
    pub max_constant_buffer_slots: u32,
}

/// O que o dispositivo criado suporta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub feature_level: FeatureLevel,
    pub is_software: bool,
    pub supports_compute: bool,
    pub supports_tessellation: bool,
    pub supports_logic_ops: bool,
}

impl DeviceCapabilities {
    pub(crate) fn from_feature_level(feature_level: FeatureLevel, is_software: bool) -> Self {
        Self {
            feature_level,
            is_software,
            supports_compute: feature_level.supports_compute(),
            supports_tessellation: feature_level.supports_tessellation(),
            supports_logic_ops: feature_level.supports_logic_ops(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_level_ordering_drives_capability_checks() {
        assert!(FeatureLevel::Level11_0.supports_tessellation());
        assert!(!FeatureLevel::Level10_1.supports_tessellation());
        assert!(FeatureLevel::Level10_0.supports_compute());
        assert!(!FeatureLevel::Level9_3.supports_compute());
        assert!(FeatureLevel::Level11_1.supports_logic_ops());
        assert!(!FeatureLevel::Level11_0.supports_logic_ops());
    }

    #[test]
    fn capabilities_derived_from_feature_level() {
        let caps = DeviceCapabilities::from_feature_level(FeatureLevel::Level11_0, false);
        assert!(caps.supports_compute);
        assert!(caps.supports_tessellation);
        assert!(!caps.supports_logic_ops);
        assert!(!caps.is_software);
    }
}
