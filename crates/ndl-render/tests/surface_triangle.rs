//! Prova de que `ndl-render` não depende de `ndl-platform` (ou de
//! qualquer outra lib de janela): este teste abre uma janela Win32 crua,
//! na mão, e entrega só o `RawWindowHandle` dela para `ndl-render`. A
//! conexão entre uma lib de janela e o render é sempre trabalho do jogo.
#![cfg(target_os = "windows")]

use std::num::NonZeroIsize;

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW,
    PostQuitMessage, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    MSG, PM_REMOVE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use ndl_render::core::{Device, DeviceDescriptor};
use ndl_render::surface::{Extent2D, Surface, Swapchain, SwapchainDescriptor};

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn create_raw_window(width: i32, height: i32) -> HWND {
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).expect("GetModuleHandleW").into();
        let class_name = w!("NdlRenderSurfaceTest");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        CreateWindowExW(
            Default::default(),
            class_name,
            w!("ndl-render - teste de surface"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            None,
            None,
            hinstance,
            None,
        )
        .expect("CreateWindowExW")
    }
}

fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[test]
fn clears_a_real_window_to_a_color() {
    let hwnd = create_raw_window(800, 600);

    let mut handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd.0 as isize).unwrap());
    handle.hinstance = None;
    let raw_handle = RawWindowHandle::Win32(handle);

    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");
    let surface = Surface::from_window_handle(raw_handle).expect("falha ao criar surface");

    let mut swapchain = Swapchain::new(
        &device,
        &surface,
        &SwapchainDescriptor {
            size: Extent2D::new(800, 600),
            ..Default::default()
        },
    )
    .expect("falha ao criar swapchain");

    // Limpa por alguns quadros com uma cor visível (azul NDL) — se isso
    // não paniciar e a janela aparecer pintada, o caminho inteiro
    // device -> surface -> swapchain -> clear -> present está funcionando.
    for _ in 0..30 {
        pump_messages();
        swapchain
            .clear_and_present(&device, [0.1, 0.2, 0.6, 1.0])
            .expect("falha ao limpar/apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

#[test]
fn clears_using_context_and_view_directly() {
    use ndl_render::context::Context;

    let hwnd = create_raw_window(640, 480);

    let mut handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd.0 as isize).unwrap());
    handle.hinstance = None;
    let raw_handle = RawWindowHandle::Win32(handle);

    let device = Device::new(&DeviceDescriptor::default()).expect("falha ao criar device");
    let surface = Surface::from_window_handle(raw_handle).expect("falha ao criar surface");

    let mut swapchain = Swapchain::new(
        &device,
        &surface,
        &SwapchainDescriptor {
            size: Extent2D::new(640, 480),
            ..Default::default()
        },
    )
    .expect("falha ao criar swapchain");

    let context: Context = device.context();

    for _ in 0..10 {
        pump_messages();
        let view = swapchain
            .render_target_view(&device)
            .expect("falha ao obter a render target view");
        context.set_render_target(view);
        context.clear_render_target(view, [0.8, 0.1, 0.1, 1.0]);
        context.flush();
        swapchain.present().expect("falha ao apresentar");
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
