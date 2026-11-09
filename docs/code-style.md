---
description: Convenções de código Rust usadas nas crates do NDL — descriptors, erros, documentação, visibilidade, operadores
scope: root
covers: [rust, style, descriptor, builder, error-handling, doc-comments, visibility, naming, derive, operators]
---

# Convenções de código

- **Descriptors de configuração**: struct simples com campos `pub`, um `impl Default` com valores
  sensatos, construído com `..Default::default()` — nunca um builder com encadeamento de métodos
  (`.with_x().with_y()`). Exemplos: `WindowDescriptor`, `DeviceDescriptor`, `SwapchainDescriptor`,
  `BufferDescriptor`.

- **Erros**: um único `#[derive(Debug, Clone, PartialEq, Eq)] pub enum XError` por crate
  (`PlatformError`, `GraphicsError`), com `impl fmt::Display` manual (`match` por variante,
  mensagem em português) e um `impl std::error::Error for XError {}` vazio — não use
  `thiserror`/`anyhow`/derive macros de erro. Variantes carregam dados estruturados
  (`InvalidArgument { name, reason }`, `OutOfBounds { resource, offset, size, limit }`) em vez de
  só uma `String` solta, sempre que quem chama pode querer dar `match` no motivo.

- **Derives em tipos de valor** (vetores, descriptors, ids, enums): no mínimo
  `Debug, Clone, Copy, PartialEq`; acrescente `Eq, Hash` quando o tipo for usado como chave de
  mapa ou comparado por igualdade em cache; marque `#[non_exhaustive]` em enums que a crate pode
  estender depois sem isso ser uma breaking change (`BackendApi`, `Format`, `PrimitiveTopology`,
  `QueryType`, ...).

- **Idioma dos comentários de documentação**: `//!` (módulo) e `///` (item) são escritos em
  português do Brasil, no mesmo tom do README — vale para todas as crates, inclusive as de
  infraestrutura pura. Mensagens de commit, ao contrário, são em inglês (confira o tom/tempo
  verbal com `git log` antes de escrever uma).

- **Seção "Estado atual"**: um módulo parcialmente implementado documenta isso explicitamente no
  seu `//!`, com uma seção `## Estado atual` dizendo o que já é real e o que continua reservado
  (ver `ndl-render::buffer` e o `//!` da raiz de `ndl-render`). Atualize essa seção conforme o
  módulo ganha funcionalidade; só a remova (dobrando o conteúdo de volta na descrição principal)
  quando o módulo estiver inteiramente implementado.

- **Visibilidade**: qualquer coisa específica de backend ou detalhe de implementação é
  `pub(crate)`, nunca `pub` — só a API portátil e agnóstica de backend é exportada pelo
  `mod.rs`/`lib.rs` do módulo. `pub use` em `lib.rs`/`mod.rs` é o único lugar onde um tipo passa a
  fazer parte da superfície pública da crate.

- **Organização de módulos**: um conceito sem submódulos é um arquivo único (`ndl-math::vec3`); um
  conceito com múltiplos arquivos é uma pasta com `mod.rs` (`ndl-platform::window::{mod.rs,
  descriptor.rs, attributes.rs, fullscreen.rs, ...}`).

- **Dependências não-óbvias no `Cargo.toml`**: qualquer dependência que vá além do propósito óbvio
  da crate leva um comentário `#` explicando o motivo — principalmente quando poderia parecer que
  quebra o isolamento entre crates (ver os comentários de `raw-window-handle` e do
  `[dev-dependencies]` `ndl-math` em `crates/ndl-render/Cargo.toml`). Nunca adicione uma
  dependência dessas em silêncio.

- **Operadores e traits em recursos de GPU**: implemente operadores bitwise de verdade
  (`BitOr`/`BitAnd`/`BitXor`/`Not` + variantes `Assign`) em tipos de flags
  (`BufferBindFlags`, `ShaderStages`, `ColorWriteMask`, ...). Nunca implemente operadores
  aritméticos (`Add`/`Mul`/`Div`) em tipos de recurso de GPU (texturas, buffers) — isso esconderia
  execução de shader/cópias caras atrás de uma sintaxe inocente. A operação equivalente deve ser
  uma função explícita (`add_images()`, `blend_images()`, ...). Para o conjunto completo de traits
  padrão planejados por tipo gráfico (`Clone`, `Debug`, `Hash`, `Drop`, `From`/`TryFrom`,
  `AsRef`/`Deref` com moderação), veja o `//!` da raiz de `ndl-render`.
