//! Sincronização CPU/GPU.
//!
//! O D3D11 não oferece, em todas as versões, o mesmo modelo de fences e
//! semáforos do Vulkan — este módulo não promete fences nativas em
//! qualquer configuração D3D11.
//!
//! ## Planejado
//!
//! API básica: `flush()`, `wait_idle()`, `is_gpu_idle()`,
//! `create_event_query()`, `signal_completion()`, `poll_completion()`,
//! `wait_for_completion()`. `wait_idle()` é construída pela crate,
//! possivelmente usando queries de evento e polling controlado.
//!
//! Extensões opcionais (interfaces D3D11 mais recentes, com suporte
//! apropriado, isoladas em um submódulo `sync::fence` com detecção
//! explícita de capacidades): `create_fence()`, `signal_fence()`,
//! `wait_fence()`, `get_completed_value()`, `set_event_on_completion()`,
//! `open_shared_fence()`.
//!
//! Ainda não implementado.
