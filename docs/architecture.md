---
description: Princípios de arquitetura do workspace NDL — isolamento entre crates, papel de cada crate, crate fachada e features
scope: root
covers: [architecture, crates, workspace, isolation, facade, features, ecs]
---

# Arquitetura do workspace NDL

- NDL é uma infraestrutura modular de crates Rust para construir jogos (espírito MonoGame/XNA) —
  **não é um editor nem uma suíte de ferramentas**. Nunca adicione uma crate ou módulo que
  pressuponha um editor, um pipeline de assets com GUI, ou qualquer coisa fora do código do jogo.

- Regra central, sem exceção: **nenhuma crate conhece o domínio de outra**. `ndl-render` não sabe
  o que é física, RPG ou qualquer lógica de jogo; `ndl-logger` não sabe o que está acontecendo em
  `ndl-physics`. Quem conecta módulos entre si é sempre o código do jogo, nunca uma crate
  dependendo de outra crate de domínio.

- Dependências de infraestrutura pura entre crates do NDL são aceitáveis em princípio (ex:
  `ndl-math` não representa "domínio de jogo", só fundação matemática compartilhada) — mas, na
  prática atual, `ndl-render` e `ndl-platform` vão além do mínimo exigido e **não têm nenhuma
  dependência de runtime a outra crate do NDL**, nem mesmo `ndl-math`: ver
  [`backend-abstraction.md`](backend-abstraction.md) e [`testing.md`](testing.md) para o motivo
  (ex: `ndl-math` só entra como `[dev-dependencies]` de `ndl-render`, para os testes montarem uma
  matriz de transform). Ao criar um módulo novo, siga esse padrão mais estrito: receba bytes crus
  ou tipos genéricos, nunca importe outra crate `ndl-*` em `[dependencies]` de uma crate de
  infraestrutura gráfica/plataforma.

- `ndl-platform` nunca depende de `ndl-render` (ou de qualquer lib de janela/gráfica) e
  vice-versa. O ponto de conexão entre os dois é um `RawWindowHandle` (crate `raw-window-handle`),
  extraído e entregue pelo código do jogo — nunca importe uma crate pela outra para "facilitar".

- Por esse mesmo motivo, o NDL **não** fornece um ECS no núcleo da engine: ECS é uma escolha de
  arquitetura de jogo, não uma responsabilidade de infraestrutura. Se um ECS vier a existir, será
  uma crate opcional e desacoplada, nunca uma dependência de `ndl-render`/`ndl-platform`/etc.

- A crate `ndl` é uma **fachada pura**: não implementa nada, só reexporta as demais crates atrás
  de feature flags (`pub use ndl_math as math;` sob `#[cfg(feature = "math")]`, etc.), com
  `default = []` — habilitar uma feature nunca deve ativar funcionalidade extra além de trazer
  aquela crate para a árvore de dependências. Para adicionar uma crate nova a essa fachada, siga
  [`../how-tos/add-new-crate.md`](../how-tos/add-new-crate.md) (ou rode
  `scripts/new-crate.py`, que já faz essa parte).

- Workspace membro = `crates/*` (root `Cargo.toml`). O diretório `playground/` é **deliberadamente
  não é membro do workspace** — ver [`playground.md`](playground.md).

- Crates hoje reais: `ndl-math`, `ndl-platform`, `ndl-render`. Crates hoje stub (só lib.rs com a
  intenção documentada, sem implementação): `ndl-config`, `ndl-dependencys`, `ndl-diagnostics`,
  `ndl-logger` — cada uma termina seu `//!` de módulo com a frase exata "Ainda não implementada:
  esta crate é um stub nesta fase do projeto."; mantenha essa frase ao criar um novo stub, para
  que seja fácil encontrar todas as crates pendentes com uma busca de texto.

- Crates planejadas para rodadas futuras (ainda sem pasta): `ndl-jobs`, `ndl-runtime`,
  `ndl-physics`, `ndl-audio`, `ndl-input` (hoje vive dentro de `ndl-platform`), `ndl-assets`,
  `ndl-anim`. Ao propor uma crate nova, confira essa lista antes de inventar um nome novo para
  responsabilidade que já tem dono planejado.
