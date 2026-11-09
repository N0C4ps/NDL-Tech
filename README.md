# NDL

NDL é um ecossistema de crates Rust para construção de jogos, no espírito do
MonoGame/XNA: **não é um editor nem uma suíte de ferramentas**, e sim uma
infraestrutura modular de crates que fornecem abstrações, métodos e
automações para quem está construindo um jogo por cima.

Cada crate é fina e focada em uma responsabilidade. Por exemplo, `ndl-render`
não implementa uma API gráfica própria — ela abstrai backends gráficos
existentes (prioritariamente [wgpu](https://wgpu.rs/), com DX12 como alvo
inicial, podendo se estender a Vulkan/Metal/GL) para aumentar a eficiência de
desenvolvimento.

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
| `ndl-render` | Abstrações de renderização sobre backends gráficos (wgpu). |
| `ndl-config` | Carregamento e parsing de configurações. |
| `ndl-dependencys` | Organização de dependências entre crates/módulos. |
| `ndl-diagnostics` | Diagnósticos e telemetria de desenvolvimento. |
| `ndl-logger` | Logging estruturado. |

Crates planejadas para rodadas futuras: `ndl-jobs`, `ndl-runtime`,
`ndl-physics`, `ndl-audio`, `ndl-platform`, `ndl-input`, `ndl-assets`,
`ndl-anim`, além de uma crate fachada `ndl` que reexporta as demais via
feature flags (ex: `use ndl::render::...`).

## Build

```sh
cargo build
cargo test
```
