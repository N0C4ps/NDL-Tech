---
description: Convenções de teste — assertivas reais em vez de "não quebrou", isolamento em dev-dependencies, testes sem GPU dedicada
scope: root
covers: [testing, integration-tests, assertions, gpu, warp, dev-dependencies, pixel]
---

# Convenções de teste

- Testes vivem na pasta `tests/` de cada crate como testes de integração (um arquivo por
  cenário/feature: `triangle.rs`, `textured_quad.rs`, `depth_and_blend.rs`, `command_queue.rs`,
  `memory_and_labels.rs`, `transfer_query_sync_debug.rs`, `compute.rs`, ...), exercitando a API
  pública como um jogo real faria — não só unidades `#[cfg(test)] mod tests` dentro do módulo
  (embora `ndl-math` use esse estilo para os tipos de vetor, que não têm estado/recursos externos).

- Testes de renderização/plataforma validam o **resultado real observado**, nunca só "não
  lançou panic/erro": `Swapchain::capture_frame` lê de volta os pixels reais do back buffer (via
  staging texture) e o teste compara contra a cor esperada calculada a partir da
  geometria/transformação/amostragem; testes de `Context::copy_buffer` leem o buffer copiado via
  `Context::map_read` e comparam byte a byte com a origem. Ao alterar um teste existente, mantenha
  (ou reforce) esse tipo de assertiva — nunca regrida para um `.expect(...)` sem checar o
  resultado.

- Um teste que precisa provar a independência de uma crate abre seus próprios recursos crus de
  plataforma em vez de importar a crate de quem precisa ficar desacoplada:
  `ndl-render::tests::{surface_triangle, triangle}` abrem uma janela Win32 crua direto (via
  `windows`/`raw-window-handle`), nunca via `ndl-platform`, provando que o render funciona com
  *qualquer* handle de janela.

- `ndl-math` pode ser usada como dependência **só em `[dev-dependencies]`** de `ndl-render`,
  exclusivamente para os testes de integração montarem uma matriz de transform com conveniência —
  nunca adicione `ndl-math` (ou qualquer outra crate `ndl-*`) em `[dependencies]` de `ndl-render`;
  código de produção só recebe bytes crus (ver `Device::create_buffer_with_data`,
  `Context::update_buffer`). Veja [`architecture.md`](architecture.md) para a regra geral de
  isolamento.

- Testes específicos de Windows/GPU levam `#[cfg(target_os = "windows")]`, com uma contrapartida
  explícita `#[cfg(not(target_os = "windows"))]` validando que a operação retorna erro em
  plataformas não suportadas (ver `device_creation_is_unsupported_outside_windows` em
  `tests/device_creation.rs`) — nunca deixe uma plataforma não-Windows simplesmente sem teste
  nenhum; valide a falha esperada.

- Para rodar os testes de `ndl-render` numa máquina/CI sem GPU dedicada, passe
  `DeviceDescriptor { prefer_software: true, ..Default::default() }` para forçar o adaptador de
  software WARP (ver `create_warp_software_device` em `tests/device_creation.rs`) em vez de pular
  o teste — veja também [`../how-tos/run-tests-without-gpu.md`](../how-tos/run-tests-without-gpu.md).
