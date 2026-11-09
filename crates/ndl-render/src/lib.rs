//! `ndl-render` fornece a infraestrutura de renderização do ecossistema NDL.
//!
//! Esta crate **não implementa uma API gráfica própria**: ela abstrai
//! backends gráficos existentes (Direct3D 11 e 12, via `backend::d3d11`/
//! `backend::d3d12` — ver `backend::Backend` —, podendo se estender a
//! Vulkan/Metal/GL depois) para aumentar a eficiência de quem constrói
//! jogos sobre o NDL.
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
//! O backend Direct3D 11 possui implementação funcional para o subconjunto
//! documentado em cada módulo. Recursos ainda não implementados retornam
//! erros tipados; a API portátil continua sendo ampliada gradualmente.
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

pub mod command;
pub mod context;
pub mod queue;

pub mod surface;

pub mod buffer;
pub mod memory;
pub mod resource;
pub mod texture;
pub mod view;

pub mod blend;
pub mod depth_stencil;
pub mod input;
pub mod pipeline;
pub mod raster;
pub mod sampler;
pub mod shader;

pub mod compute;
pub mod render;
pub mod transfer;

pub mod debug;
pub mod format;
pub mod query;
pub mod sync;

mod backend;
