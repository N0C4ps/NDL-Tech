use ndl_render::core::{Device, DeviceDescriptor, Instance, LimitsSource};

#[test]
#[cfg(target_os = "windows")]
fn create_default_hardware_device() {
    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");

    let caps = device.capabilities();
    println!("backend: {:?}", device.backend_api());
    println!("is_software: {}", device.is_software());
    println!("supports_compute: {}", caps.supports_compute());

    assert!(!device.is_lost());
    assert_eq!(device.limits_source(), LimitsSource::ApiGuarantee);

    let limits = device.limits();
    assert!(limits.max_texture_dimension_2d >= 2048);
}

#[test]
#[cfg(target_os = "windows")]
fn create_warp_software_device() {
    let descriptor = DeviceDescriptor {
        prefer_software: true,
        ..Default::default()
    };
    let device = Device::new(&descriptor).expect("falha ao criar device WARP");
    assert!(device.is_software());
}

#[test]
#[cfg(target_os = "windows")]
fn enumerate_adapters_finds_at_least_one() {
    let instance = Instance::new().expect("instance init");
    let adapters = instance
        .enumerate_adapters()
        .expect("falha ao enumerar adaptadores");

    assert!(
        !adapters.is_empty(),
        "esperava pelo menos um adaptador gráfico"
    );
    for adapter in &adapters {
        println!(
            "adapter: {} (vendor=0x{:04x} device=0x{:04x} software={} vram={}MB)",
            adapter.name,
            adapter.vendor_id,
            adapter.device_id,
            adapter.is_software,
            adapter.dedicated_video_memory / (1024 * 1024)
        );
    }
}

#[test]
#[cfg(not(target_os = "windows"))]
fn device_creation_is_unsupported_outside_windows() {
    let result = Device::new(&DeviceDescriptor::default());
    assert!(result.is_err());
}
