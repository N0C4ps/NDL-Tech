---
description: Como compilar e rodar o sandbox playground/ localmente
scope: playground
covers: [playground, run, cargo-run, sandbox]
---

# Rodar o playground

1. O `playground/` é um projeto Cargo separado do workspace principal (ver
   [`../docs/playground.md`](../docs/playground.md)) — entre na pasta antes de rodar qualquer
   comando Cargo nele, não rode `cargo run -p ndl-playground` da raiz.

   ```sh
   cd playground
   cargo run
   ```

2. Por padrão isso compila contra o que está em `main` no GitHub (dependência git da crate `ndl`),
   não contra suas mudanças locais. Se você acabou de editar uma crate em `crates/` e quer ver o
   efeito no playground sem fazer push primeiro, siga
   [`test-local-changes-in-playground.md`](test-local-changes-in-playground.md) antes deste passo.

3. A execução tem duas fases, nessa ordem (ver `playground/src/main.rs`):
   - Fase headless: roda contra `PlatformBackendKind::Headless`, só imprime `[OK]`/`[FALHOU]`
     para uma série de asserts lógicos (janela, cursor, monitores, input, event loop) — não abre
     janela nenhuma. Se algo aparecer como `[FALHOU]`, há uma regressão a investigar antes de
     seguir para a fase nativa.
   - Fase nativa: abre uma janela real (backend `Native`, `winit` no Windows) e conecta
     `ndl-render` a ela via `RawWindowHandle`, desenhando um quad cuja cor varia lentamente —
     confirmação visual de que render + platform funcionam juntos de verdade.

4. Para encerrar, feche a janela nativa (dispara `WindowEvent::CloseRequested` → o processo
   termina sozinho). Não mate o processo pelo terminal como primeira opção — se o fechar da janela
   não encerrar o processo, é um sinal de regressão no ciclo de vida de `ndl-platform`, não um
   comportamento esperado.
