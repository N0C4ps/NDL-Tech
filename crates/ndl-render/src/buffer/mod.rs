//! Buffers gráficos: vertex, index, constant, structured.
//!
//! ## Planejado
//!
//! Tipos: `Buffer`, `BufferDescriptor`, `BufferUsage`, `BufferBindFlags`,
//! `BufferMapMode`, `BufferView`, `VertexBuffer`, `IndexBuffer`,
//! `ConstantBuffer`, `StructuredBuffer`, `RawBuffer`, `IndirectBuffer`,
//! `StagingBuffer`.
//!
//! ```ignore
//! impl Device {
//!     fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Buffer>;
//!     fn create_buffer_with_data(&self, desc: &BufferDescriptor, data: &[u8]) -> Result<Buffer>;
//! }
//!
//! impl Buffer {
//!     fn size(&self) -> u64;
//!     fn usage(&self) -> BufferUsage;
//!     fn bind_flags(&self) -> BufferBindFlags;
//!     fn stride(&self) -> Option<u32>;
//!     fn descriptor(&self) -> &BufferDescriptor;
//!     fn set_label(&self, label: &str);
//! }
//! ```
//!
//! Operações de buffer (executadas pelo contexto/encoder): `write_buffer()`,
//! `update_buffer()`, `update_buffer_region()`, `copy_buffer()`,
//! `copy_buffer_region()`, `read_buffer()` (via staging + sincronização,
//! não leitura direta), `map_buffer()`, `unmap_buffer()`, `discard_buffer()`,
//! `copy_structure_count()`.
//!
//! Flags de uso planejadas:
//!
//! | Categoria | Valores |
//! |---|---|
//! | Uso de memória | Default, Immutable, Dynamic, Staging |
//! | Bind | Vertex, Index, Constant, ShaderResource, UnorderedAccess, StreamOutput |
//! | Acesso CPU | Read, Write |
//! | Misc | Structured, Raw, DrawIndirectArgs, Shared, SharedKeyedMutex |
//! | Map | Read, Write, ReadWrite, WriteDiscard, WriteNoOverwrite |
//!
//! Para um sistema de alocação de alto nível, ver também
//! `crate::memory` (`UploadArena`, `ConstantBufferRing`, `StagingPool`).
//!
//! Ainda não implementado.
