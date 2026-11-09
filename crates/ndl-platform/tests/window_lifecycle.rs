use ndl_platform::core::{Platform, PlatformConfig};
use ndl_platform::window::WindowDescriptor;

#[test]
fn create_window_with_defaults() {
    let platform = Platform::new(PlatformConfig::default()).expect("platform init");
    let window = platform
        .create_window(WindowDescriptor::default())
        .expect("window creation");

    assert_eq!(window.title(), "NDL");
    assert!(window.is_visible());
    assert!(!window.is_minimized());
}

#[test]
fn window_methods_mutate_local_state() {
    let platform = Platform::new(PlatformConfig::default()).expect("platform init");
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
    let platform = Platform::new(PlatformConfig::default()).expect("platform init");
    let mut window = platform
        .create_window(WindowDescriptor::default())
        .expect("window creation");

    assert!(window.cursor().is_visible());
    window.cursor_mut().set_visible(false);
    assert!(!window.cursor().is_visible());
}
