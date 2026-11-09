---
description: Padrão de abstração de backend (SO em ndl-platform, API gráfica em ndl-render) via módulo privado + contrato interno
scope: root
covers: [backend, trait, platform-abstraction, graphics-backend, directx, winit, stub, cfg]
---

# Padrão de abstração de backend

Tanto `ndl-platform` (abstração de SO) quanto `ndl-render` (abstração de API gráfica) seguem o
mesmo padrão estrutural para esconder detalhes de implementação específicos de plataforma:

- Todo código específico de SO/API vive num módulo **inteiramente privado** chamado `backend`
  (`mod backend;` na raiz da crate — nunca `pub mod backend;`). Nada desse módulo aparece na API
  pública.

- O resto da crate programa apenas contra um contrato interno, nunca contra um backend concreto:
  em `ndl-platform` isso é um `pub(crate) trait Backend` (ver `src/backend/mod.rs`) implementado
  por cada backend; em `ndl-render` é um conjunto de funções livres em `backend::mod.rs` que fazem
  o dispatch (`enumerate_adapters()`, etc.) para o backend concreto ativo.

- A seleção do backend concreto é feita com `#[cfg(target_os = "...")]`, sempre com um fallback
  explícito em vez de um caminho que simplesmente não compila: `ndl-platform::backend::create`
  cai para `HeadlessBackend` quando não há backend nativo para o SO atual; o dispatch de
  `ndl-render::backend` retorna `GraphicsError::Unsupported(...)` em vez de tentar compilar código
  inexistente.

- Um backend ainda não implementado continua reservado na árvore de módulos (nunca é omitido do
  código-fonte) e é marcado `#[allow(dead_code)]` até ganhar implementação real — ver
  `ndl-platform::backend::{linux, macos}`. Ao reservar um módulo assim, não delete o stub quando
  parecer "morto"; ele existe para já fixar o nome/posição na árvore.

- Um backend por sistema operacional/API gráfica, cada um na sua própria pasta: hoje
  `ndl-platform::backend::{headless, windows}` são reais e `linux::{x11,wayland}`, `macos` são
  stubs; em `ndl-render::backend` só `directx` é real (Direct3D 11, via `windows-rs`), reservado
  para `vulkan`/`metal`/`opengl` depois seguindo o mesmo padrão — não implemente dois backends no
  mesmo arquivo/módulo nem misture lógica de dois backends numa função só.

- Campos ou funções que só vão ser usados por um módulo-irmão ainda não implementado levam
  `#[allow(dead_code)] // usado por <módulo> quando existir` em vez de serem apagados — sempre
  nomeie no comentário qual módulo futuro vai consumir aquele código (ver `RawDevice` em
  `ndl-render::backend::directx::mod.rs`).

- Todo backend gráfico deve expor uma forma de forçar um adaptador de software (hoje:
  `DeviceDescriptor::prefer_software` → `D3D_DRIVER_TYPE_WARP` no D3D11), para que testes e CI
  consigam rodar sem GPU dedicada — ver [`testing.md`](testing.md).
