/// Informações descritivas de um adaptador gráfico (GPU) disponível no
/// sistema. Os dados preenchidos dependem do que o backend e o driver
/// conseguem consultar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory: u64,
    pub shared_system_memory: u64,
    pub is_software: bool,
    pub device_type: AdapterType,
    /// Versão do driver, quando fornecida pelo backend/driver ativo.
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterType {
    Hardware,
    Software,
    Cpu,
    Unknown,
}
