//! Backend Direct3D 11, via `windows-rs` (crate `windows`).
//!
//! Ainda não implementado — este é o primeiro backend real planejado para
//! `ndl-render`, da mesma forma que `winit` foi o primeiro backend real
//! de `ndl-platform`.
//!
//! ## Recursos avançados do D3D11 considerados para o futuro
//!
//! | Módulo | Funcionalidades |
//! |---|---|
//! | `interop` | Recursos compartilhados, handles DXGI, keyed mutex |
//! | `video` | Video device, decoder, processor, encoder (conforme interfaces disponíveis) |
//! | `tiled` | Tiled resources, mapeamento e atualização de tiles |
//! | `stream_output` | Geometry shader stream output, buffers e contadores |
//! | `indirect` | Buffers de argumentos e execução indireta |
//! | `predication` | Renderização condicional baseada em consultas |
//! | `multithreading` | Proteção multithread, deferred contexts |
//! | `dxgi` | Adaptadores, outputs, modos de exibição, formatos |
//! | `debug` | InfoQueue, debug layer, annotations |
//! | `interop_d3d` | Acesso controlado aos objetos COM nativos |
//!
//! Interop planejado (funções `unsafe`, com contratos de ownership e
//! segurança rigorosos onde exponham/recebam ponteiros COM):
//! `raw_device()`, `raw_context()`, `raw_resource()`, `from_raw_device()`,
//! `from_raw_resource()`, `open_shared_resource()`, `get_shared_handle()`,
//! `acquire_keyed_mutex()`, `release_keyed_mutex()`.
//!
//! Tiled resources planejado: `update_tile_mappings()`,
//! `copy_tile_mappings()`, `copy_tiles()`, `update_tiles()`,
//! `resize_tile_pool()`, `tiled_resource_barrier()`.
//!
//! Multithreading planejado: `create_deferred_context()`,
//! `finish_command_list()`, `execute_command_list()`,
//! `set_multithread_protected()`, `get_multithread_protected()` — o
//! contexto imediato exige cuidado especial com acesso concorrente.
