// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

use core::ffi::c_void;
use i_slint_core::api::{PhysicalSize, Window};
use i_slint_core::graphics::{IntSize, Rgb8Pixel};
use i_slint_core::platform::Platform;
use i_slint_core::renderer::Renderer;
use i_slint_core::software_renderer::SoftwareRenderer;
use i_slint_core::window::ffi::WindowAdapterRcOpaque;
use i_slint_core::window::{WindowAdapter, WindowAdapterSealed};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::rc::Rc;

type WindowAdapterUserData = *mut c_void;

// FIXME wrapper over &dyn Renderer
#[repr(C)]
pub struct RendererPtr {
    _a: *const c_void,
    _b: *const c_void,
}

pub struct CppWindowAdapter {
    window: Window,
    user_data: WindowAdapterUserData,
    drop: unsafe extern "C" fn(WindowAdapterUserData),
    /// Safety: the returned pointer must live for the lifetime of self
    get_renderer_ref: unsafe extern "C" fn(WindowAdapterUserData) -> RendererPtr,
    show: unsafe extern "C" fn(WindowAdapterUserData),
    hide: unsafe extern "C" fn(WindowAdapterUserData),
    request_redraw: unsafe extern "C" fn(WindowAdapterUserData),
}

impl Drop for CppWindowAdapter {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.user_data) };
    }
}

impl WindowAdapter for CppWindowAdapter {
    fn window(&self) -> &Window {
        &self.window
    }
}

impl WindowAdapterSealed for CppWindowAdapter {
    fn renderer(&self) -> &dyn Renderer {
        unsafe { core::mem::transmute((self.get_renderer_ref)(self.user_data)) }
    }

    fn show(&self) {
        unsafe { (self.show)(self.user_data) }
    }
    fn hide(&self) {
        unsafe { (self.hide)(self.user_data) }
    }

    fn request_redraw(&self) {
        unsafe { (self.request_redraw)(self.user_data) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn slint_window_adapter_new(
    user_data: WindowAdapterUserData,
    drop: unsafe extern "C" fn(WindowAdapterUserData),
    get_renderer_ref: unsafe extern "C" fn(WindowAdapterUserData) -> RendererPtr,
    show: unsafe extern "C" fn(WindowAdapterUserData),
    hide: unsafe extern "C" fn(WindowAdapterUserData),
    request_redraw: unsafe extern "C" fn(WindowAdapterUserData),
    target: *mut WindowAdapterRcOpaque,
) {
    let window = Rc::<CppWindowAdapter>::new_cyclic(|w| CppWindowAdapter {
        window: Window::new(w.clone()),
        user_data,
        drop,
        get_renderer_ref,
        show,
        request_redraw,
        hide,
    });

    core::ptr::write(target as *mut Rc<dyn WindowAdapter>, window);
}

type PlatformUserData = *mut c_void;

struct CppPlatform {
    user_data: PlatformUserData,
    drop: unsafe extern "C" fn(PlatformUserData),
    window_factory: unsafe extern "C" fn(PlatformUserData, *mut WindowAdapterRcOpaque),
}

impl Drop for CppPlatform {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.user_data) };
    }
}

impl Platform for CppPlatform {
    fn create_window_adapter(&self) -> Rc<dyn WindowAdapter> {
        let mut uninit = core::mem::MaybeUninit::<Rc<dyn WindowAdapter>>::uninit();
        unsafe {
            (self.window_factory)(
                self.user_data,
                uninit.as_mut_ptr() as *mut WindowAdapterRcOpaque,
            );
            uninit.assume_init()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn slint_platform_register(
    user_data: PlatformUserData,
    drop: unsafe extern "C" fn(PlatformUserData),
    window_factory: unsafe extern "C" fn(PlatformUserData, *mut WindowAdapterRcOpaque),
) {
    let p = CppPlatform { user_data, drop, window_factory };
    i_slint_core::platform::set_platform(Box::new(p)).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn slint_windowrc_has_active_animations(
    handle: *const WindowAdapterRcOpaque,
) -> bool {
    let window_adapter = &*(handle as *const Rc<dyn WindowAdapter>);
    window_adapter.window().has_active_animations()
}

#[no_mangle]
pub extern "C" fn slint_platform_update_timers_and_animations() {
    i_slint_core::platform::update_timers_and_animations()
}

type SoftwareRendererOpaque = *const c_void;

#[no_mangle]
pub unsafe extern "C" fn slint_software_renderer_new(
    buffer_age: u32,
    window: &WindowAdapterRcOpaque,
) -> SoftwareRendererOpaque {
    let window = core::mem::transmute::<&WindowAdapterRcOpaque, &Rc<dyn WindowAdapter>>(window);
    let weak = Rc::downgrade(window);
    match buffer_age {
        0 => Box::into_raw(Box::new(SoftwareRenderer::<0>::new(weak))) as SoftwareRendererOpaque,
        1 => Box::into_raw(Box::new(SoftwareRenderer::<1>::new(weak))) as SoftwareRendererOpaque,
        2 => Box::into_raw(Box::new(SoftwareRenderer::<2>::new(weak))) as SoftwareRendererOpaque,
        _ => unreachable!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn slint_software_renderer_drop(buffer_age: u32, r: SoftwareRendererOpaque) {
    match buffer_age {
        0 => drop(Box::from_raw(r as *mut SoftwareRenderer<0>)),
        1 => drop(Box::from_raw(r as *mut SoftwareRenderer<1>)),
        2 => drop(Box::from_raw(r as *mut SoftwareRenderer<2>)),
        _ => unreachable!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn slint_software_renderer_render_rgb8(
    buffer_age: u32,
    r: SoftwareRendererOpaque,
    buffer: *mut Rgb8Pixel,
    buffer_len: usize,
    buffer_stride: usize,
) {
    let buffer = core::slice::from_raw_parts_mut(buffer, buffer_len);
    match buffer_age {
        0 => (*(r as *const SoftwareRenderer<0>)).render(buffer, buffer_stride),
        1 => (*(r as *const SoftwareRenderer<1>)).render(buffer, buffer_stride),
        2 => (*(r as *const SoftwareRenderer<2>)).render(buffer, buffer_stride),
        _ => unreachable!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn slint_software_renderer_handle(
    buffer_age: u32,
    r: SoftwareRendererOpaque,
) -> RendererPtr {
    let r = match buffer_age {
        0 => (r as *const SoftwareRenderer<0>) as *const dyn Renderer,
        1 => (r as *const SoftwareRenderer<1>) as *const dyn Renderer,
        2 => (r as *const SoftwareRenderer<2>) as *const dyn Renderer,
        _ => unreachable!(),
    };
    core::mem::transmute(r)
}

type SkiaRendererOpaque = *const c_void;
type SkiaRenderer = i_slint_renderer_skia::SkiaRenderer<CppRawHandle>;

struct CppRawHandle(RawWindowHandle, RawDisplayHandle);
// Safety: the C++ code should ensure that the handle is valid
unsafe impl raw_window_handle::HasRawWindowHandle for CppRawHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}
// Safety: the C++ code should ensure that the handle is valid
unsafe impl raw_window_handle::HasRawDisplayHandle for CppRawHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.1
    }
}

// the raw handle type are #[non_exhaustive], so they can't be initialize with the convenient syntax. Work that around.
macro_rules! init_raw {
    ($ty:ty { $($var:ident),* }) => {
        {
            let mut h = <$ty>::empty();
            $(h.$var = $var;)*
            h
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_new(
    window: &WindowAdapterRcOpaque,
) -> SkiaRendererOpaque {
    let window = core::mem::transmute::<&WindowAdapterRcOpaque, &Rc<dyn WindowAdapter>>(window);
    let weak = Rc::downgrade(window);
    Box::into_raw(Box::new(SkiaRenderer::new(weak))) as SkiaRendererOpaque
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_drop(r: SkiaRendererOpaque) {
    drop(Box::from_raw(r as *mut SkiaRenderer))
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_show_win32(
    r: SkiaRendererOpaque,
    hwnd: *mut c_void,
    hinstance: *mut c_void,
    size: IntSize,
) {
    let r = &*(r as *const SkiaRenderer);
    let handle = CppRawHandle(
        RawWindowHandle::Win32(init_raw!(raw_window_handle::Win32WindowHandle { hwnd, hinstance })),
        RawDisplayHandle::Windows(raw_window_handle::WindowsDisplayHandle::empty()),
    );
    r.show(handle, PhysicalSize { width: size.width, height: size.height })
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_show_x11(
    r: SkiaRendererOpaque,
    window: u32,
    visual_id: u32,
    connection: *mut c_void,
    screen: core::ffi::c_int,
    size: IntSize,
) {
    use raw_window_handle::{XcbDisplayHandle, XcbWindowHandle};
    let r = &*(r as *const SkiaRenderer);
    let handle = CppRawHandle(
        RawWindowHandle::Xcb(init_raw!(XcbWindowHandle { window, visual_id })),
        RawDisplayHandle::Xcb(init_raw!(XcbDisplayHandle { connection, screen })),
    );
    r.show(handle, PhysicalSize { width: size.width, height: size.height })
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_show_wayland(
    r: SkiaRendererOpaque,
    surface: *mut c_void,
    display: *mut c_void,
    size: IntSize,
) {
    use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};

    let r = &*(r as *const SkiaRenderer);
    let handle = CppRawHandle(
        RawWindowHandle::Wayland(init_raw!(WaylandWindowHandle { surface })),
        RawDisplayHandle::Wayland(init_raw!(WaylandDisplayHandle { display })),
    );
    r.show(handle, PhysicalSize { width: size.width, height: size.height })
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_show_appkit(
    r: SkiaRendererOpaque,
    ns_view: *mut c_void,
    ns_window: *mut c_void,
    size: IntSize,
) {
    use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle};

    let r = &*(r as *const SkiaRenderer);
    let handle = CppRawHandle(
        RawWindowHandle::AppKit(init_raw!(AppKitWindowHandle { ns_view, ns_window })),
        RawDisplayHandle::AppKit(AppKitDisplayHandle::empty()),
    );
    r.show(handle, PhysicalSize { width: size.width, height: size.height })
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_hide(r: SkiaRendererOpaque) {
    let r = &*(r as *const SkiaRenderer);
    r.hide()
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_resize(r: SkiaRendererOpaque, size: IntSize) {
    let r = &*(r as *const SkiaRenderer);
    r.resize_event(PhysicalSize { width: size.width, height: size.height });
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_render(r: SkiaRendererOpaque, size: IntSize) {
    let r = &*(r as *const SkiaRenderer);
    r.render(PhysicalSize { width: size.width, height: size.height });
}

#[no_mangle]
pub unsafe extern "C" fn slint_skia_renderer_handle(r: SkiaRendererOpaque) -> RendererPtr {
    let r = (r as *const SkiaRenderer) as *const dyn Renderer;
    core::mem::transmute(r)
}
