---
description: Como rodar os testes de ndl-render numa máquina ou CI sem GPU dedicada
scope: ndl-render
covers: [testing, gpu, warp, software-rendering, ci]
---

# Rodar testes de `ndl-render` sem GPU dedicada

A maioria dos testes de integração de `ndl-render` cria um `Device` de verdade contra uma GPU de
hardware por padrão (`DeviceDescriptor::default()`). Numa máquina/runner de CI sem GPU dedicada,
isso falha na criação do device, não no teste em si.

1. Em qualquer teste/trecho que crie um device, passe `prefer_software: true` em vez do descriptor
   padrão:

   ```rust
   use ndl_render::core::{Device, DeviceDescriptor};

   let device = Device::new(&DeviceDescriptor {
       prefer_software: true,
       ..Default::default()
   })
   .expect("falha ao criar device WARP");
   ```

   Isso força o driver tipo `D3D_DRIVER_TYPE_WARP` (adaptador de software do Direct3D) em vez de
   `D3D_DRIVER_TYPE_HARDWARE` — ver `create_warp_software_device` em
   `crates/ndl-render/tests/device_creation.rs` para um exemplo já existente.

2. Confirme que funcionou checando `device.capabilities().is_software` — deve ser `true`.

3. Não pule (`#[ignore]`) um teste só porque a máquina atual não tem GPU — prefira adaptar o
   teste para usar `prefer_software: true` (ou, se o teste especificamente precisa validar
   hardware real, mantenha-o gated por `#[cfg(target_os = "windows")]` como já é hoje, e documente
   no próprio teste que ele exige hardware).

4. Testes que não envolvem device nenhum (ex: `ndl-math`) não são afetados por isso — essa
   convenção só vale para os testes de `ndl-render` que criam um `Device`.
