//! Mapeamento, staging e upload.
//!
//! Separa duas coisas: acesso CPU a recursos do backend (mapeamento) e
//! gerenciamento de alocações de alto nível da própria crate (arenas de
//! upload, pools de staging). Não é uma API de heaps explícitos — o
//! D3D11 gerencia grande parte da memória via runtime/driver.
//!
//! ## Planejado
//!
//! Métodos de mapeamento: `map()`, `map_read()`, `map_write()`,
//! `map_read_write()`, `map_write_discard()`, `map_write_no_overwrite()`,
//! `unmap()`, `write_mapped()`, `read_mapped()`, `mapped_row_pitch()`,
//! `mapped_depth_pitch()`.
//!
//! Uso pretendido com guarda RAII:
//!
//! ```ignore
//! let mut mapped = context.map_write_discard(&buffer)?;
//! mapped.copy_from_slice(&data);
//! // Unmap automaticamente no Drop
//! ```
//!
//! `MappedResource` deve guardar o empréstimo necessário para impedir uso
//! incompatível do contexto/recurso enquanto estiver mapeado.
//!
//! Alocadores de alto nível planejados: `allocate()`, `allocate_aligned()`,
//! `allocate_transient()`, `free()`, `reset()`, `reserve()`, `capacity()`,
//! `used_bytes()`, `available_bytes()` — para tipos como `UploadArena`,
//! `DynamicBufferAllocator`, `StagingPool`, `TransientResourcePool`,
//! `ConstantBufferRing`, `ResourceCache`.
//!
//! Ainda não implementado.
