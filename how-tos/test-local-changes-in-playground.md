---
description: Como validar uma mudança local (ainda não pushed) através do playground
scope: playground
covers: [playground, local, path-dependency, git-dependency]
---

# Testar mudanças locais no playground

O playground sempre depende de `ndl` via URL git (ver
[`../docs/playground.md`](../docs/playground.md)), então por padrão ele nunca vê uma mudança que
você só tem localmente. Para testar uma mudança antes de fazer push:

1. Rode `python3 scripts/playground_use_local.py on` — isso troca a dependência `ndl` em
   `playground/Cargo.toml` de `git = "..."` para `path = "../crates/ndl"`, preservando as mesmas
   `features` já habilitadas.
2. Rode `python3 scripts/playground_use_local.py status` se quiser confirmar qual modo está ativo
   antes de compilar.
3. Siga [`run-the-playground.md`](run-the-playground.md) normalmente (`cd playground && cargo
   run`) — agora ele compila contra o código local em `crates/`.
4. **Antes de comitar qualquer coisa**, rode `python3 scripts/playground_use_local.py off` para
   voltar à dependência git — mesmo que `playground/` esteja no `.gitignore`, deixar o path local
   ativo significa que qualquer outra pessoa (ou você, numa outra máquina) rodando o playground
   vai tentar resolver um `path = "../crates/ndl"` que só existe no seu checkout.
5. Se preferir não reverter manualmente, rode `python3 scripts/playground_use_local.py status`
   antes de fechar a sessão de trabalho para confirmar que voltou para `git` — trate isso como
   parte do checklist de "terminei de testar", igual fechar a janela nativa do playground.
