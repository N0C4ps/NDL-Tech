//! Dispositivo, adaptadores e capacidades.
//!
//! Responsável por inicializar o backend gráfico, descobrir a GPU
//! disponível e expor o que o hardware/driver suporta — a base de onde
//! todo o resto da crate parte.
//!
//! ## Planejado
//!
//! Tipos: `Instance`, `Adapter`, `Device`, `DeviceDescriptor`,
//! `DeviceCapabilities`, `DeviceLimits`, `FeatureLevel`, `AdapterInfo`,
//! `GraphicsError`.
//!
//! Métodos essenciais em `Adapter`/`Device`: `info()`, `is_software()`,
//! `create_device()`, `capabilities()`, `limits()`, `is_lost()`.
//!
//! Outros métodos úteis: `adapter_luid()`, `vendor_id()`, `device_id()`,
//! `dedicated_video_memory()`, `shared_system_memory()`,
//! `driver_description()`, `shader_model()`, `max_texture_dimension()`,
//! `max_render_targets()`, `max_anisotropy()`, `max_uav_slots()`,
//! `supports_compute()`, `supports_tessellation()`, `supports_logic_ops()`.
//!
//! Ainda não implementado — ver `backend::directx` para onde a
//! implementação real (criação de `ID3D11Device`, enumeração de
//! adaptadores via DXGI) vai entrar.
