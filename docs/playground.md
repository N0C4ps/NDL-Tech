---
description: Propósito e regras do diretório playground — sandbox isolado que consome o NDL como dependência externa real
scope: playground
covers: [playground, sandbox, git-dependency, workspace-isolation, gitignore]
---

# O diretório `playground/`

- `playground/` é um sandbox de smoke-test manual para exercitar a API pública como um consumidor
  externo de verdade exercitaria — não é uma ferramenta de desenvolvimento do workspace, e não é
  coberto por `cargo test` do workspace principal.

- Ele declara sua própria tabela `[workspace]` vazia em `playground/Cargo.toml` exatamente para o
  Cargo **não** absorvê-lo no workspace raiz (`members = ["crates/*"]`) — ele é um projeto Cargo
  totalmente separado, com seu próprio `Cargo.lock`.

- Ele depende da crate fachada `ndl` via **URL git**
  (`ndl = { git = "https://github.com/N0C4ps/NDL-Tech.git", features = [...] }`), nunca via
  `path = "../crates/ndl"` — o objetivo é provar que a crate funciona do jeito que qualquer jogo
  puxando ela do GitHub usaria, incluindo a superfície de reexport por feature flag.

- Consequência direta: `cargo run` dentro de `playground/` compila contra o que estiver em
  `main` no GitHub, não contra suas mudanças locais não commitadas/pushed. Para validar uma
  mudança local através do playground, veja
  [`../how-tos/test-local-changes-in-playground.md`](../how-tos/test-local-changes-in-playground.md)
  (ou rode `scripts/playground-use-local.py on`) — e sempre revertar (`off`) antes de comitar.

- `playground/` inteiro está no `.gitignore` da raiz — é um sandbox local, nunca deve ser
  versionado nem tratado como exemplo/documentação oficial da crate.

- Ao editar `playground/src/main.rs`, mantenha a estrutura em duas fases do `main()`: primeiro o
  backend `Headless` com asserts via `check()` (sem janela real, cobre a API logicamente), depois
  o backend `Native` com uma janela real + swapchain (verificação visual humana). Novas features
  habilitadas na fachada devem ganhar cobertura nas duas fases, não só numa.
