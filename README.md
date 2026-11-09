# NDL

NDL é um ecossistema de crates Rust para construção de jogos, no espírito do
MonoGame/XNA: **não é um editor nem uma suíte de ferramentas**, e sim uma
infraestrutura modular de crates que fornecem abstrações, métodos e
automações para quem está construindo um jogo por cima.

Cada crate é fina e focada em uma responsabilidade. Por exemplo, `ndl-render`
não implementa uma API gráfica própria — ela abstrai backends gráficos
existentes (prioritariamente Direct3D 11 nesta fase, podendo se estender a
Vulkan/Metal/OpenGL depois) para aumentar a eficiência de desenvolvimento.

## Princípio de arquitetura: isolamento entre crates

Cada crate do NDL é **independente** e não conhece o domínio das outras:

- `ndl-render` não sabe nada sobre física, RPG ou qualquer lógica do jogo que
  a está usando.
- `ndl-logger` não sabe o que está acontecendo em `ndl-physics`.
- Quem conecta os módulos entre si é o **código do jogo**, não as crates.

Dependências puramente técnicas de infraestrutura são aceitáveis — por
exemplo, `ndl-render` pode depender de `ndl-math` para seus tipos de vetores
e matrizes, pois `ndl-math` não representa "domínio de jogo", e sim fundação
matemática compartilhada.

Por esse mesmo motivo, o NDL **não** fornece um ECS no núcleo da engine: ECS
é uma escolha de arquitetura de jogo, não uma responsabilidade de
infraestrutura. Se um ECS vier a ser oferecido, será uma crate opcional e
desacoplada, nunca uma dependência de `ndl-render`/`ndl-physics`/etc.

## Crates

| Crate | Responsabilidade |
|---|---|
| `ndl-math` | Vetores, matrizes e quaternions — fundação matemática compartilhada. |
| `ndl-platform` | Abstração sobre o SO: janelas, eventos, input, cursor, monitores. |
| `ndl-render` | Abstrações de renderização sobre backends gráficos (Direct3D 11, depois outros). |
| `ndl-config` | Carregamento e parsing de configurações. |
| `ndl-dependencys` | Organização de dependências entre crates/módulos. |
| `ndl-diagnostics` | Diagnósticos e telemetria de desenvolvimento. |
| `ndl-logger` | Logging estruturado. |

`ndl-platform` oferece a interface pública completa (janela, eventos, input,
cursor, monitores), com os backends `headless` e `windows` (via `winit`)
funcionais — os backends `linux/x11`, `linux/wayland` e `macos` existem na
árvore de módulos como stubs reservados para a integração real com cada SO.

Os 25 módulos planejados de `ndl-render` já existem na árvore. Vários têm
implementação funcional contra o backend Direct3D 11, incluindo device/
adapter, surface/swapchain, buffers, shaders HLSL (vertex/pixel/compute),
texturas/sampler, pipelines gráficos com estado fixo explícito, depth/blend, gravação diferida e fila lógica,
cópias, queries, sincronização e debug. Isso descreve o subconjunto já
implementado, não uma API completa: cada módulo documenta seus recursos
disponíveis e pendências, e partes como múltiplos backends, shaders
geometry/hull/domain e operações de inspeção continuam reservadas.

Desenho instanciado (`Context::draw_instanced`/`draw_indexed_instanced`,
com atributos de vertex buffer por instância via
`VertexAttribute::step_mode`) e desenho/dispatch indireto
(`Context::draw_indirect`/`draw_indexed_indirect`/`dispatch_indirect`, lendo
os argumentos de um `Buffer` com `BufferUsage::Indirect` em vez de recebê-los
da CPU a cada chamada) já são reais — a base para desenhar muitos objetos
parecidos numa única chamada e para GPU-driven rendering. Por ora um buffer
indirect só é escrito pela CPU; gravá-lo via UAV num compute shader continua
reservado.

MRT (multiple render targets) também é real: `Context::set_render_targets`/
`set_render_targets_with_depth` vinculam de um a
`DeviceLimits::max_color_attachments` alvos de cor simultâneos, e
`RenderPassDescriptor::colors`/`GraphicsPipelineStateDescriptor::color_formats`
expõem isso no nível do `RenderPass` (um elemento = caso comum, vários =
MRT — ex: deferred shading escrevendo albedo/normal num só pass).
`Device::create_render_target_texture` cria uma textura off-screen pronta
para servir de attachment extra, e `Device::capture_texture` lê de volta o
conteúdo de qualquer textura (o mesmo caminho de
`Swapchain::capture_frame`, generalizado) — o jeito usado pelos testes para
confirmar que cada alvo recebeu o valor certo, não só que o pass não
quebrou. O `BlendState` único de um pipeline já se aplica a todos os alvos
igualmente (D3D11 sem `IndependentBlendEnable`); blend independente por
alvo continua reservado.

Pipelines podem declarar formatos esperados de cor e depth; o `RenderPass`
confere esses formatos e dimensões antes de emitir draws. O estado fixo atual
inclui rasterização, blend (incluindo alpha separado e máscara de canais) e
depth/stencil com operações por face; stencil reference é definido no
descritor do pipeline.

Os testes de renderização não só checam "não quebrou" — `Swapchain::capture_frame`
lê de volta os pixels reais do back buffer (via staging texture), e os
testes comparam contra o resultado esperado calculado a partir da
geometria/transformação/amostragem de textura. Isso já pegou um erro real
de teste (um ponto de verificação caindo exatamente sobre um vértice, com
área zero) — a validação é sensível, não decorativa. O mesmo vale para
`Context::copy_buffer`: um teste lê de volta o buffer copiado via
`Context::map_read` e compara byte a byte com o original, em vez de só
confiar que `CopyResource` não lançou erro.

`CommandEncoder` reaproveita a API pública de `Context` por baixo (via
`Deref`), mas mantém um contexto diferido e estado de bindings próprios.
Handles de `Device::context()` compartilham o contexto imediato; as chamadas
e submissões da crate são serializadas individualmente. Uma sequência de
várias chamadas imediatas ainda pode ser intercalada entre threads; grave
sequências indivisíveis num `CommandEncoder` e submeta-as pela fila.

Testado com um triângulo colorido (`tests/triangle.rs`), um quad
texturizado que pulsa de tamanho via uma matriz de `ndl-math` enviada por
um constant buffer (`tests/textured_quad.rs`) — `ndl-render` nunca depende
de `ndl-math`; é o jogo (no teste) que monta os bytes da matriz —, dois
triângulos opacos sobrepostos em profundidades diferentes mais um quad
semi-transparente por cima, tudo via `draw_indexed` (`tests/depth_and_blend.rs`),
e um triângulo desenhado via `CommandEncoder`/`GraphicsQueue::submit`/`wait_idle`
em vez do contexto imediato direto (`tests/command_queue.rs`).

`Device::new()` cria um dispositivo Direct3D 11 de verdade (via
`backend::directx`, `windows-rs`), com `DeviceCapabilities`/`DeviceLimits`
traduzidos pelo backend para capacidades e limites portaveis;
`Instance::enumerate_adapters()` lista
as GPUs do sistema via DXGI; `Surface`/`Swapchain` conectam um
`RawWindowHandle` (crate `raw-window-handle`) a uma swapchain real.

**`ndl-render` nunca depende de `ndl-platform`** (nem de nenhuma outra lib
de janela) — `Surface::from_window_handle` recebe um `RawWindowHandle`
genérico; é sempre o código do jogo que extrai esse handle de onde a janela
foi criada e o entrega ao render. O teste `tests/surface_triangle.rs` prova
isso abrindo uma janela Win32 crua, sem tocar em `ndl-platform`.

Por padrão, `Swapchain::present` usa `PresentMode::VSync` — "liso" numa GPU
topo de linha não significa nada numa GPU mais modesta, então a crate nunca
assume um present sem limite de quadros como padrão.

## Crate fachada `ndl`

A crate `ndl` reexporta as demais crates via feature flags, para quem
prefere depender de um único pacote:

```toml
[dependencies]
ndl = { git = "https://github.com/N0C4ps/NDL-Tech.git", features = ["math", "platform"] }
```

```rust
use ndl::platform::core::{Platform, PlatformConfig};
use ndl::math::Vec3;
```

Habilitar uma feature só traz aquela crate para a árvore de dependências —
nenhuma funcionalidade extra é ativada por padrão (`default = []`). Features
disponíveis hoje: `math`, `platform`, `render`, `config`, `dependencys`,
`diagnostics`, `logger`, e `full` (todas juntas).

Crates planejadas para rodadas futuras: `ndl-jobs`, `ndl-runtime`,
`ndl-physics`, `ndl-audio`, `ndl-input` (hoje parte de `ndl-platform`),
`ndl-assets`, `ndl-anim`.

## Build

```sh
cargo build
cargo test
```
