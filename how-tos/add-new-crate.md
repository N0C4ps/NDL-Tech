---
description: Passo a passo para adicionar uma nova crate ndl-xxx ao workspace e à fachada ndl
scope: root
covers: [crate, workspace, facade, scaffold, new-crate]
---

# Adicionar uma nova crate ao workspace

1. Confira [`../docs/architecture.md`](../docs/architecture.md) — a lista de crates planejadas
   (`ndl-jobs`, `ndl-runtime`, `ndl-physics`, `ndl-audio`, `ndl-input`, `ndl-assets`, `ndl-anim`)
   já tem nome e responsabilidade reservados; não invente um nome novo para algo que já está
   nessa lista.

2. Rode `scripts/new_crate.py <nome> "<descrição>"` (prefira `--dry-run` primeiro para revisar o
   que vai ser escrito). Em nomes/descrições com acento, se o texto sair corrompido nos arquivos
   gerados, rode de novo com `PYTHONUTF8=1` na frente (`PYTHONUTF8=1 python3 scripts/new_crate.py
   ...`) — veja o aviso no `--help` do script. Isso já faz, de uma vez:
   - cria `crates/<nome>/Cargo.toml` (herdando `workspace.package`) e
     `crates/<nome>/src/lib.rs` com o template padrão de crate stub (ver
     [`../docs/architecture.md`](../docs/architecture.md));
   - adiciona `<nome>` como dependência opcional em `crates/ndl/Cargo.toml`;
   - adiciona a feature correspondente (nome sem o prefixo `ndl-`) em `crates/ndl/Cargo.toml`,
     inclusive na lista `full = [...]`;
   - adiciona o `#[cfg(feature = "...")] pub use ndl_xxx as xxx;` correspondente em
     `crates/ndl/src/lib.rs`.

3. Passo manual que o script **não** faz: adicione uma linha na tabela de crates do `README.md`
   (raiz do repo) com o nome e a responsabilidade da nova crate.

4. Rode `python3 scripts/check_crate_isolation.py` para confirmar que a crate nova não introduziu
   nenhuma dependência de runtime a outra crate `ndl-*` fora de `[dev-dependencies]` — a nova
   crate deve nascer tão isolada quanto as demais (ver
   [`../docs/architecture.md`](../docs/architecture.md)).

5. Rode `cargo build` na raiz do workspace para confirmar que tudo ainda compila, incluindo a
   crate `ndl` com a feature nova habilitada: `cargo build -p ndl --features <feature>`.

6. Se a crate nova vai ser exercitada pelo playground, siga
   [`run-the-playground.md`](run-the-playground.md) — lembre que o playground só vê crates que já
   foram pushed para `main` no GitHub (ou que você apontou localmente, ver
   [`test-local-changes-in-playground.md`](test-local-changes-in-playground.md)).
