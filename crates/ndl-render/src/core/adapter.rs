/// Informações descritivas de um adaptador gráfico (GPU) disponível no
/// sistema, obtidas via enumeração de adaptadores do backend (DXGI, no
/// caso do Direct3D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory: u64,
    pub shared_system_memory: u64,
    pub is_software: bool,
}
