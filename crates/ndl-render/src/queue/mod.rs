//! Fila lógica e submissão.
//!
//! O D3D11 não tem filas gráficas, de computação e de cópia separadas
//! como o D3D12/Vulkan — este módulo oferece uma `GraphicsQueue` lógica
//! única que abstrai a submissão de comandos sobre esse modelo, em vez de
//! fingir que existem três filas nativas independentes.
//!
//! ## Planejado
//!
//! ```ignore
//! impl GraphicsQueue {
//!     fn submit(&mut self, commands: &[CommandBuffer]) -> Result<()>;
//!     fn flush(&mut self);
//!     fn wait_idle(&mut self) -> Result<()>;
//! }
//! ```
//!
//! `submit()` abstrai a execução de comandos do D3D11; `wait_idle()`
//! precisa de queries de evento e polling controlado, não de um simples
//! `Flush()`.
//!
//! Ainda não implementado.
