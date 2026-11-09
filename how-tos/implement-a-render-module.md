---
description: Passo a passo para tirar um módulo reservado de ndl-render do papel e implementá-lo contra o backend Direct3D 11
scope: ndl-render
covers: [ndl-render, backend, directx, module, implementation]
---

# Implementar um módulo reservado de `ndl-render`

Todos os 25 módulos planejados de `ndl-render` já existem na árvore (`pub mod ...;` em
`crates/ndl-render/src/lib.rs`); implementar um deles segue sempre a mesma sequência (ver o
histórico de commits do crate para exemplos reais, um módulo por commit):

1. Leia o `//!` do módulo-alvo (`crates/ndl-render/src/<modulo>/mod.rs`) — a seção
   "## Estado atual", quando existir, já diz o que falta. Se o módulo ainda não tem essa seção,
   confirme no `//!` da raiz da crate (`crates/ndl-render/src/lib.rs`) o que foi planejado para
   ele.

2. Implemente a contraparte no backend concreto, em
   `crates/ndl-render/src/backend/directx/mod.rs` (hoje o único backend real) — nunca no módulo
   público diretamente. Siga [`../docs/backend-abstraction.md`](../docs/backend-abstraction.md):
   o módulo público chama o backend através do contrato interno, nunca referencia tipos `windows`/
   D3D11 fora de `backend::directx`.

3. Siga as convenções de [`../docs/code-style.md`](../docs/code-style.md) para os novos tipos
   públicos: descriptor com `Default` + `..Default::default()`, derives mínimos
   (`Debug, Clone, Copy, PartialEq`, mais `Eq`/`Hash`/`#[non_exhaustive]` quando fizer sentido),
   variantes de erro estruturadas em `GraphicsError` se o módulo precisar de um novo tipo de falha,
   e documentação em português.

4. Nunca importe outra crate `ndl-*` em `[dependencies]` para implementar o módulo — veja
   [`../docs/architecture.md`](../docs/architecture.md); receba sempre bytes crus ou tipos
   genéricos do chamador.

5. Escreva o teste de integração em `crates/ndl-render/tests/<cenario>.rs`, com assertivas sobre
   resultado real (pixels capturados via `Swapchain::capture_frame`, ou dados lidos de volta via
   `Context::map_read`), nunca só "não lançou erro" — ver
   [`../docs/testing.md`](../docs/testing.md). Se o teste criar um `Device`, considere também
   cobrir [`run-tests-without-gpu.md`](run-tests-without-gpu.md) quando fizer sentido.

6. Atualize a seção "## Estado atual" do `//!` do módulo (ou do `//!` raiz da crate, se for o
   último módulo reservado) para refletir o que passou a ser real — não deixe a documentação
   dizer "reservado" sobre algo que já foi implementado.

7. Rode `cargo test -p ndl-render` (numa máquina Windows com o SDK do Direct3D) antes de
   considerar o módulo pronto.
