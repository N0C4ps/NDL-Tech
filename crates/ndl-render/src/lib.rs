//! `ndl-render` fornece a infraestrutura de renderização do ecossistema NDL.
//!
//! Esta crate **não implementa uma API gráfica própria**: ela abstrai
//! backends gráficos existentes (prioritariamente Direct3D via `backend::directx`
//! nesta fase, podendo se estender a Vulkan/Metal/GL depois) para aumentar a
//! eficiência de quem constrói jogos sobre o NDL.
//!
//! Assim como as demais crates do NDL, `ndl-render` não conhece nada sobre
//! física, áudio, ECS ou qualquer outra lógica de jogo. Ela também não
//! carrega texturas do disco por conta própria — essa é responsabilidade de
//! `ndl-assets`; o que `ndl-render` faz é receber dados já carregados e
//! produzir os recursos de GPU (texturas, buffers) a partir deles. A conexão
//! entre `ndl-render` e `ndl-assets` é feita pelo código do jogo, não por
//! uma depender da outra.
//!
//! ## Estado atual
//!
//! Esta é a fase de **reserva de arquitetura**: cada módulo abaixo existe
//! com a responsabilidade documentada e os tipos/métodos planejados, mas a
//! implementação real (começando pelo backend Direct3D) ainda está por
//! vir. Nenhum destes módulos é funcional ainda.
//!
//! ## Convenções planejadas de operadores e traits
//!
//! Para flags (`BufferBindFlags`, `ShaderStages`, `ColorWriteMask`, ...),
//! usar operadores bitwise de verdade (`BitOr`, `BitOrAssign`, `BitAnd`,
//! `BitAndAssign`, `BitXor`, `BitXorAssign`, `Not`) — ex:
//! `BufferBindFlags::VERTEX | BufferBindFlags::SHADER_RESOURCE`.
//!
//! Traits padrão a implementar nos tipos gráficos, quando fizer sentido:
//! `Clone` (handles compartilhados com ownership seguro), `Debug`
//! (inspeção de descriptors), `PartialEq`/`Eq` (comparação de handles e
//! enums), `Hash` (cache de estados/pipelines), `Default` (descriptors
//! padrão), `Drop` (liberação de recursos e guards), `From`/`TryFrom`
//! (conversões entre enums Rust e os do backend), `AsRef`/`Deref` com
//! moderação (só para wrappers realmente transparentes).
//!
//! Deliberadamente **não** planejado: `Add`/`Mul`/`Div` em recursos de GPU
//! (ex: `texture_a + texture_b`) — isso esconderia execução de shaders e
//! criaria operações caras de forma implícita. Operações de imagem devem
//! ser funções explícitas (`add_images()`, `blend_images()`, ...).

pub mod core;

pub mod context;
pub mod command;
pub mod queue;

pub mod surface;

pub mod resource;
pub mod memory;
pub mod buffer;
pub mod texture;
pub mod view;

pub mod shader;
pub mod pipeline;
pub mod input;
pub mod raster;
pub mod blend;
pub mod depth_stencil;
pub mod sampler;

pub mod render;
pub mod compute;
pub mod transfer;

pub mod query;
pub mod sync;
pub mod debug;
pub mod format;

mod backend;
