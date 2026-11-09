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

`ndl-render` tem a árvore de módulos inteira reservada (`context`, `command`,
`queue`, `resource`, `memory`, `buffer`, `texture`, `view`, `shader`,
`pipeline`, `input`, `raster`, `blend`, `depth_stencil`, `sampler`, `render`,
`compute`, `transfer`, `query`, `sync`, `debug`), cada uma documentada com
os tipos e métodos planejados — mas ainda sem implementação.

`ndl-render::core`, `ndl-render::format` e `ndl-render::surface` já são
**reais**: `Device::new()` cria um dispositivo Direct3D 11 de verdade (via
`backend::directx`, `windows-rs`), com `DeviceCapabilities`/`DeviceLimits`
derivados do `FeatureLevel` obtido; `Instance::enumerate_adapters()` lista
as GPUs do sistema via DXGI; `Format` cobre as principais famílias de pixel
format com conversão para/do `DXGI_FORMAT` nativo; e `Surface`/`Swapchain`
conectam um `RawWindowHandle` (crate `raw-window-handle`) a uma swapchain
real, com `clear_and_present` já limpando e apresentando quadros numa
janela de verdade.

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
