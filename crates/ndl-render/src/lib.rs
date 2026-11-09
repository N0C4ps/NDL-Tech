//! `ndl-render` fornece abstrações de renderização para o ecossistema NDL.
//!
//! Esta crate não implementa uma API gráfica própria: ela abstrai backends
//! gráficos existentes (prioritariamente wgpu, com DX12 como alvo principal,
//! podendo se estender a Vulkan/Metal/GL) para aumentar a eficiência de quem
//! constrói jogos sobre o NDL. Não conhece nada sobre física, áudio ou
//! qualquer outro domínio de jogo — apenas renderização.
//!
//! Ainda não implementada: esta crate é um stub nesta fase do projeto.
