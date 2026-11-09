use ndl_platform::core::{Platform, PlatformBackendKind, PlatformConfig};
use ndl_platform::window::WindowDescriptor;

/// Estes testes exercitam o caminho leve de criação de janela
/// (`Platform::create_window`), que só existe no backend `headless` —
/// o backend nativo real abre janela de SO via `Platform::run_window`.
fn headless_platform() -> Platform {
    Platform::new(PlatformConfig {
        backend: PlatformBackendKind::Headless,
        ..Default::default()
    })
    .expect("platform init")
}

#[test]
fn create_window_with_defaults() {
    let platform = headless_platform();
    let window = platform
        .create_window(WindowDescriptor::default())
        .expect("window creation");

    assert_eq!(window.title(), "NDL");
    assert!(window.is_visible());
    assert!(!window.is_minimized());
}

#[test]
fn window_methods_mutate_local_state() {
    let platform = headless_platform();
    let mut window = platform
        .create_window(WindowDescriptor::default())
        .expect("window creation");

    window.set_title("Outro título");
    assert_eq!(window.title(), "Outro título");

    window.hide();
    assert!(!window.is_visible());
    window.show();
    assert!(window.is_visible());

    window.maximize();
    assert!(window.is_maximized());
    window.restore();
    assert!(!window.is_maximized());
}

#[test]
fn cursor_defaults_to_visible_default_icon() {
    let platform = headless_platform();
    let mut window = platform
        .create_window(WindowDescriptor::default())
        .expect("window creation");

    assert!(window.cursor().is_visible());
    window.cursor_mut().set_visible(false);
    assert!(!window.cursor().is_visible());
}
