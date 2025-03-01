// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

/*!
This module contains types that are public and re-exported in the slint-rs as well as the slint-interpreter crate as public API.
*/

#![warn(missing_docs)]

use alloc::boxed::Box;

use crate::component::ComponentVTable;
use crate::input::{KeyEventType, KeyInputEvent, MouseEvent};
use crate::window::{WindowAdapter, WindowInner};

/// A position represented in the coordinate space of logical pixels. That is the space before applying
/// a display device specific scale factor.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct LogicalPosition {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
}

impl LogicalPosition {
    /// Construct a new logical position from the given x and y coordinates, that are assumed to be
    /// in the logical coordinate space.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert a given physical position to a logical position by dividing the coordinates with the
    /// specified scale factor.
    pub fn from_physical(physical_pos: PhysicalPosition, scale_factor: f32) -> Self {
        Self::new(physical_pos.x as f32 / scale_factor, physical_pos.y as f32 / scale_factor)
    }

    /// Convert this logical position to a physical position by multiplying the coordinates with the
    /// specified scale factor.
    pub fn to_physical(&self, scale_factor: f32) -> PhysicalPosition {
        PhysicalPosition::from_logical(*self, scale_factor)
    }

    pub(crate) fn to_euclid(&self) -> crate::lengths::LogicalPoint {
        [self.x as _, self.y as _].into()
    }
}

/// A position represented in the coordinate space of physical device pixels. That is the space after applying
/// a display device specific scale factor to pixels from the logical coordinate space.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct PhysicalPosition {
    /// The x coordinate.
    pub x: i32,
    /// The y coordinate.
    pub y: i32,
}

impl PhysicalPosition {
    /// Construct a new physical position from the given x and y coordinates, that are assumed to be
    /// in the physical coordinate space.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert a given logical position to a physical position by multiplying the coordinates with the
    /// specified scale factor.
    pub fn from_logical(logical_pos: LogicalPosition, scale_factor: f32) -> Self {
        Self::new((logical_pos.x * scale_factor) as i32, (logical_pos.y * scale_factor) as i32)
    }

    /// Convert this physical position to a logical position by dividing the coordinates with the
    /// specified scale factor.
    pub fn to_logical(&self, scale_factor: f32) -> LogicalPosition {
        LogicalPosition::from_physical(*self, scale_factor)
    }

    #[cfg(feature = "ffi")]
    pub(crate) fn to_euclid(&self) -> crate::graphics::euclid::default::Point2D<i32> {
        [self.x, self.y].into()
    }
}

/// The position of the window in either physical or logical pixels. This is used
/// with [`Window::set_position`].
#[derive(Clone, Debug, derive_more::From, PartialEq)]
pub enum WindowPosition {
    /// The position in physical pixels.
    Physical(PhysicalPosition),
    /// The position in logical pixels.
    Logical(LogicalPosition),
}

impl WindowPosition {
    /// Turn the `WindowPosition` into a `PhysicalPosition`.
    pub fn to_physical(&self, scale_factor: f32) -> PhysicalPosition {
        match self {
            WindowPosition::Physical(pos) => pos.clone(),
            WindowPosition::Logical(pos) => pos.to_physical(scale_factor),
        }
    }
}

/// A size represented in the coordinate space of logical pixels. That is the space before applying
/// a display device specific scale factor.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct LogicalSize {
    /// The width in logical pixels.
    pub width: f32,
    /// The height in logical.
    pub height: f32,
}

impl LogicalSize {
    /// Construct a new logical size from the given width and height values, that are assumed to be
    /// in the logical coordinate space.
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert a given physical size to a logical size by dividing width and height by the
    /// specified scale factor.
    pub fn from_physical(physical_size: PhysicalSize, scale_factor: f32) -> Self {
        Self::new(
            physical_size.width as f32 / scale_factor,
            physical_size.height as f32 / scale_factor,
        )
    }

    /// Convert this logical size to a physical size by multiplying width and height with the
    /// specified scale factor.
    pub fn to_physical(&self, scale_factor: f32) -> PhysicalSize {
        PhysicalSize::from_logical(*self, scale_factor)
    }

    pub(crate) fn to_euclid(&self) -> crate::lengths::LogicalSize {
        [self.width as _, self.height as _].into()
    }
}

/// A size represented in the coordinate space of physical device pixels. That is the space after applying
/// a display device specific scale factor to pixels from the logical coordinate space.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct PhysicalSize {
    /// The width in physical pixels.
    pub width: u32,
    /// The height in physical pixels;
    pub height: u32,
}

impl PhysicalSize {
    /// Construct a new physical size from the width and height values, that are assumed to be
    /// in the physical coordinate space.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Convert a given logical size to a physical size by multiplying width and height with the
    /// specified scale factor.
    pub fn from_logical(logical_size: LogicalSize, scale_factor: f32) -> Self {
        Self::new(
            (logical_size.width * scale_factor) as u32,
            (logical_size.height * scale_factor) as u32,
        )
    }

    /// Convert this physical size to a logical size by dividing width and height by the
    /// specified scale factor.
    pub fn to_logical(&self, scale_factor: f32) -> LogicalSize {
        LogicalSize::from_physical(*self, scale_factor)
    }

    #[cfg(feature = "ffi")]
    pub(crate) fn to_euclid(&self) -> crate::graphics::euclid::default::Size2D<u32> {
        [self.width, self.height].into()
    }
}

/// The size of a window represented in either physical or logical pixels. This is used
/// with [`Window::set_size`].
#[derive(Clone, Debug, derive_more::From, PartialEq)]
pub enum WindowSize {
    /// The size in physical pixels.
    Physical(PhysicalSize),
    /// The size in logical screen pixels.
    Logical(LogicalSize),
}

impl WindowSize {
    /// Turn the `WindowSize` into a `PhysicalSize`.
    pub fn to_physical(&self, scale_factor: f32) -> PhysicalSize {
        match self {
            WindowSize::Physical(size) => size.clone(),
            WindowSize::Logical(size) => size.to_physical(scale_factor),
        }
    }

    /// Turn the `WindowSize` into a `LogicalSize`.
    pub fn to_logical(&self, scale_factor: f32) -> LogicalSize {
        match self {
            WindowSize::Physical(size) => size.to_logical(scale_factor),
            WindowSize::Logical(size) => size.clone(),
        }
    }
}

/// This enum describes a low-level access to specific graphics APIs used
/// by the renderer.
#[derive(Clone)]
#[non_exhaustive]
pub enum GraphicsAPI<'a> {
    /// The rendering is done using OpenGL.
    NativeOpenGL {
        /// Use this function pointer to obtain access to the OpenGL implementation - similar to `eglGetProcAddress`.
        get_proc_address: &'a dyn Fn(&str) -> *const core::ffi::c_void,
    },
    /// The rendering is done on a HTML Canvas element using WebGL.
    WebGL {
        /// The DOM element id of the HTML Canvas element used for rendering.
        canvas_element_id: &'a str,
        /// The drawing context type used on the HTML Canvas element for rendering. This is the argument to the
        /// `getContext` function on the HTML Canvas element.
        context_type: &'a str,
    },
}

impl<'a> core::fmt::Debug for GraphicsAPI<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GraphicsAPI::NativeOpenGL { .. } => write!(f, "GraphicsAPI::NativeOpenGL"),
            GraphicsAPI::WebGL { context_type, .. } => {
                write!(f, "GraphicsAPI::WebGL(context_type = {})", context_type)
            }
        }
    }
}

/// This enum describes the different rendering states, that will be provided
/// to the parameter of the callback for `set_rendering_notifier` on the `slint::Window`.
#[derive(Debug, Clone)]
#[repr(C)]
#[non_exhaustive]
pub enum RenderingState {
    /// The window has been created and the graphics adapter/context initialized. When OpenGL
    /// is used for rendering, the context will be current.
    RenderingSetup,
    /// The scene of items is about to be rendered.  When OpenGL
    /// is used for rendering, the context will be current.
    BeforeRendering,
    /// The scene of items was rendered, but the back buffer was not sent for display presentation
    /// yet (for example GL swap buffers). When OpenGL is used for rendering, the context will be current.
    AfterRendering,
    /// The window will be destroyed and/or graphics resources need to be released due to other
    /// constraints.
    RenderingTeardown,
}

/// Internal trait that's used to map rendering state callbacks to either a Rust-API provided
/// impl FnMut or a struct that invokes a C callback and implements Drop to release the closure
/// on the C++ side.
pub trait RenderingNotifier {
    /// Called to notify that rendering has reached a certain state.
    fn notify(&mut self, state: RenderingState, graphics_api: &GraphicsAPI);
}

impl<F: FnMut(RenderingState, &GraphicsAPI)> RenderingNotifier for F {
    fn notify(&mut self, state: RenderingState, graphics_api: &GraphicsAPI) {
        self(state, graphics_api)
    }
}

/// This enum describes the different error scenarios that may occur when the application
/// registers a rendering notifier on a [`crate::Window`](struct.Window.html).
#[derive(Debug, Clone)]
#[repr(C)]
#[non_exhaustive]
pub enum SetRenderingNotifierError {
    /// The rendering backend does not support rendering notifiers.
    Unsupported,
    /// There is already a rendering notifier set, multiple notifiers are not supported.
    AlreadySet,
}

/// This type represents a window towards the windowing system, that's used to render the
/// scene of a component. It provides API to control windowing system specific aspects such
/// as the position on the screen.
#[repr(transparent)]
pub struct Window(pub(crate) WindowInner);

/// This enum describes whether a Window is allowed to be hidden when the user tries to close the window.
/// It is the return type of the callback provided to [Window::on_close_requested].
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[repr(C)]
pub enum CloseRequestResponse {
    /// The Window will be hidden (default action)
    #[default]
    HideWindow,
    /// The close request is rejected and the window will be kept shown.
    KeepWindowShown,
}

impl Window {
    /// Create a new window from a window adapter
    ///
    /// You only need to create the window yourself when you create a
    /// [`WindowAdapter`](crate::platform::WindowAdapter) from
    /// [`Platform::create_window_adapter`](crate::platform::Platform::create_window_adapter)
    ///
    /// Since the window adapter must own the Window, this function is meant to be used with
    /// [`Rc::new_cyclic`](alloc::rc::Rc::new_cyclic)
    ///
    /// # Example
    /// ```rust
    /// use std::rc::Rc;
    /// use slint::platform::WindowAdapter;
    /// use slint::Window;
    /// struct MyWindowAdapter {
    ///     window: Window,
    ///     //...
    /// }
    /// impl WindowAdapter for MyWindowAdapter {
    ///    fn window(&self) -> &Window { &self.window }
    ///    //...
    /// }
    /// # impl i_slint_core::window::WindowAdapterSealed for MyWindowAdapter {
    /// #   fn renderer(&self) -> &dyn i_slint_core::renderer::Renderer { unimplemented!() }
    /// # }
    ///
    /// fn create_window_adapter() -> Rc<dyn WindowAdapter> {
    ///    Rc::<MyWindowAdapter>::new_cyclic(|weak| {
    ///        MyWindowAdapter {
    ///           window: Window::new(weak.clone()),
    ///           //...
    ///        }
    ///    })
    /// }
    /// ```
    #[doc(hidden)]
    pub fn new(window_adapter_weak: alloc::rc::Weak<dyn WindowAdapter>) -> Self {
        Self(WindowInner::new(window_adapter_weak))
    }

    /// Registers the window with the windowing system in order to make it visible on the screen.
    pub fn show(&self) {
        self.0.show();
    }

    /// De-registers the window from the windowing system, therefore hiding it.
    pub fn hide(&self) {
        self.0.hide();
    }

    /// This function allows registering a callback that's invoked during the different phases of
    /// rendering. This allows custom rendering on top or below of the scene.
    pub fn set_rendering_notifier(
        &self,
        callback: impl FnMut(RenderingState, &GraphicsAPI) + 'static,
    ) -> Result<(), SetRenderingNotifierError> {
        self.0.window_adapter().renderer().set_rendering_notifier(Box::new(callback))
    }

    /// This function allows registering a callback that's invoked when the user tries to close a window.
    /// The callback has to return a [CloseRequestResponse].
    pub fn on_close_requested(&self, callback: impl FnMut() -> CloseRequestResponse + 'static) {
        self.0.on_close_requested(callback);
    }

    /// This function issues a request to the windowing system to redraw the contents of the window.
    pub fn request_redraw(&self) {
        self.0.window_adapter().request_redraw();
    }

    /// This function returns the scale factor that allows converting between logical and
    /// physical pixels.
    pub fn scale_factor(&self) -> f32 {
        self.0.scale_factor()
    }

    /// Returns the position of the window on the screen, in physical screen coordinates and including
    /// a window frame (if present).
    pub fn position(&self) -> PhysicalPosition {
        self.0.window_adapter().position()
    }

    /// Sets the position of the window on the screen, in physical screen coordinates and including
    /// a window frame (if present).
    /// Note that on some windowing systems, such as Wayland, this functionality is not available.
    pub fn set_position(&self, position: impl Into<WindowPosition>) {
        let position = position.into();
        self.0.window_adapter().set_position(position)
    }

    /// Returns the size of the window on the screen, in physical screen coordinates and excluding
    /// a window frame (if present).
    pub fn size(&self) -> PhysicalSize {
        self.0.inner_size.get()
    }

    /// Resizes the window to the specified size on the screen, in physical pixels and excluding
    /// a window frame (if present).
    pub fn set_size(&self, size: impl Into<WindowSize>) {
        let size = size.into();
        let l = size.to_logical(self.scale_factor()).to_euclid();
        let p = size.to_physical(self.scale_factor());

        self.0.set_window_item_geometry(l);
        if self.0.inner_size.replace(p) != p {
            self.0.window_adapter().set_size(size);
        }
    }

    /// Dispatch a window event to the scene.
    ///
    /// Use this when you're implementing your own backend and want to forward user input events.
    ///
    /// Any position fields in the event must be in the logical pixel coordinate system relative to
    /// the top left corner of the window.
    pub fn dispatch_event(&self, event: crate::platform::WindowEvent) {
        match event {
            crate::platform::WindowEvent::PointerPressed { position, button } => {
                self.0.process_mouse_input(MouseEvent::Pressed {
                    position: position.to_euclid().cast(),
                    button,
                });
            }
            crate::platform::WindowEvent::PointerReleased { position, button } => {
                self.0.process_mouse_input(MouseEvent::Released {
                    position: position.to_euclid().cast(),
                    button,
                });
            }
            crate::platform::WindowEvent::PointerMoved { position } => {
                self.0.process_mouse_input(MouseEvent::Moved {
                    position: position.to_euclid().cast(),
                });
            }
            crate::platform::WindowEvent::PointerScrolled { position, delta_x, delta_y } => {
                self.0.process_mouse_input(MouseEvent::Wheel {
                    position: position.to_euclid().cast(),
                    delta_x,
                    delta_y,
                });
            }
            crate::platform::WindowEvent::PointerExited => {
                self.0.process_mouse_input(MouseEvent::Exit)
            }

            crate::platform::WindowEvent::KeyPressed { text } => {
                self.0.process_key_input(KeyInputEvent {
                    text: SharedString::from(text),
                    event_type: KeyEventType::KeyPressed,
                    ..Default::default()
                })
            }
            crate::platform::WindowEvent::KeyReleased { text } => {
                self.0.process_key_input(KeyInputEvent {
                    text: SharedString::from(text),
                    event_type: KeyEventType::KeyReleased,
                    ..Default::default()
                })
            }
        }
    }

    /// Returns true if there is an animation currently active on any property in the Window; false otherwise.
    pub fn has_active_animations(&self) -> bool {
        // TODO make it really per window.
        crate::animations::CURRENT_ANIMATION_DRIVER.with(|driver| driver.has_active_animations())
    }

    /// Returns the visibility state of the window. This function can return false even if you previously called show()
    /// on it, for example if the user minimized the window.
    pub fn is_visible(&self) -> bool {
        self.0.window_adapter().is_visible()
    }
}

pub use crate::SharedString;

/// This trait is used to obtain references to global singletons exported in `.slint`
/// markup. Alternatively, you can use [`ComponentHandle::global`] to obtain access.
///
/// This trait is implemented by the compiler for each global singleton that's exported.
///
/// # Example
/// The following example of `.slint` markup defines a global singleton called `Palette`, exports
/// it and modifies it from Rust code:
/// ```rust
/// # i_slint_backend_testing::init();
/// slint::slint!{
/// export global Palette := {
///     property<color> foreground-color;
///     property<color> background-color;
/// }
///
/// export App := Window {
///    background: Palette.background-color;
///    Text {
///       text: "Hello";
///       color: Palette.foreground-color;
///    }
///    // ...
/// }
/// }
/// let app = App::new();
/// app.global::<Palette>().set_background_color(slint::Color::from_rgb_u8(0, 0, 0));
///
/// // alternate way to access the global singleton:
/// Palette::get(&app).set_foreground_color(slint::Color::from_rgb_u8(255, 255, 255));
/// ```
///
/// See also the [language reference for global singletons](docs/langref/index.html#global-singletons) for more information.
///
/// **Note:** Only globals that are exported or re-exported from the main .slint file will
/// be exposed in the API
pub trait Global<'a, Component> {
    /// Returns a reference that's tied to the life time of the provided component.
    fn get(component: &'a Component) -> Self;
}

/// This trait describes the common public API of a strongly referenced Slint component.
/// It allows creating strongly-referenced clones, a conversion into/ a weak pointer as well
/// as other convenience functions.
///
/// This trait is implemented by the [generated component](mod@crate#generated-components)
pub trait ComponentHandle {
    /// The type of the generated component.
    #[doc(hidden)]
    type Inner;
    /// Returns a new weak pointer.
    fn as_weak(&self) -> Weak<Self>
    where
        Self: Sized;

    /// Returns a clone of this handle that's a strong reference.
    #[must_use]
    fn clone_strong(&self) -> Self;

    /// Internal function used when upgrading a weak reference to a strong one.
    #[doc(hidden)]
    fn from_inner(_: vtable::VRc<ComponentVTable, Self::Inner>) -> Self;

    /// Marks the window of this component to be shown on the screen. This registers
    /// the window with the windowing system. In order to react to events from the windowing system,
    /// such as draw requests or mouse/touch input, it is still necessary to spin the event loop,
    /// using [`crate::run_event_loop`](fn.run_event_loop.html).
    fn show(&self);

    /// Marks the window of this component to be hidden on the screen. This de-registers
    /// the window from the windowing system and it will not receive any further events.
    fn hide(&self);

    /// Returns the Window associated with this component. The window API can be used
    /// to control different aspects of the integration into the windowing system,
    /// such as the position on the screen.
    fn window(&self) -> &Window;

    /// This is a convenience function that first calls [`Self::show`], followed by [`crate::run_event_loop()`](fn.run_event_loop.html)
    /// and [`Self::hide`].
    fn run(&self);

    /// This function provides access to instances of global singletons exported in `.slint`.
    /// See [`Global`] for an example how to export and access globals from `.slint` markup.
    fn global<'a, T: Global<'a, Self>>(&'a self) -> T
    where
        Self: Sized;
}

mod weak_handle {

    use super::*;

    /// Struct that's used to hold weak references of a [Slint component](mod@crate#generated-components)
    ///
    /// In order to create a Weak, you should use [`ComponentHandle::as_weak`].
    ///
    /// Strong references should not be captured by the functions given to a lambda,
    /// as this would produce a reference loop and leak the component.
    /// Instead, the callback function should capture a weak component.
    ///
    /// The Weak component also implement `Send` and can be send to another thread.
    /// but the upgrade function will only return a valid component from the same thread
    /// as the one it has been created from.
    /// This is useful to use with [`invoke_from_event_loop()`] or [`Self::upgrade_in_event_loop()`].
    pub struct Weak<T: ComponentHandle> {
        inner: vtable::VWeak<ComponentVTable, T::Inner>,
        #[cfg(feature = "std")]
        thread: std::thread::ThreadId,
    }

    impl<T: ComponentHandle> Clone for Weak<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                #[cfg(feature = "std")]
                thread: self.thread,
            }
        }
    }

    impl<T: ComponentHandle> Weak<T> {
        #[doc(hidden)]
        pub fn new(rc: &vtable::VRc<ComponentVTable, T::Inner>) -> Self {
            Self {
                inner: vtable::VRc::downgrade(rc),
                #[cfg(feature = "std")]
                thread: std::thread::current().id(),
            }
        }

        /// Returns a new strongly referenced component if some other instance still
        /// holds a strong reference. Otherwise, returns None.
        ///
        /// This also returns None if the current thread is not the thread that created
        /// the component
        pub fn upgrade(&self) -> Option<T>
        where
            T: ComponentHandle,
        {
            #[cfg(feature = "std")]
            if std::thread::current().id() != self.thread {
                return None;
            }
            self.inner.upgrade().map(T::from_inner)
        }

        /// Convenience function that returns a new strongly referenced component if
        /// some other instance still holds a strong reference and the current thread
        /// is the thread that created this component.
        /// Otherwise, this function panics.
        pub fn unwrap(&self) -> T {
            self.upgrade().unwrap()
        }

        /// Convenience function that combines [`invoke_from_event_loop()`] with [`Self::upgrade()`]
        ///
        /// The given functor will be added to an internal queue and will wake the event loop.
        /// On the next iteration of the event loop, the functor will be executed with a `T` as an argument.
        ///
        /// If the component was dropped because there are no more strong reference to the component,
        /// the functor will not be called.
        ///
        /// # Example
        /// ```rust
        /// # i_slint_backend_testing::init();
        /// slint::slint! { MyApp := Window { property <int> foo; /* ... */ } }
        /// let handle = MyApp::new();
        /// let handle_weak = handle.as_weak();
        /// let thread = std::thread::spawn(move || {
        ///     // ... Do some computation in the thread
        ///     let foo = 42;
        ///     # assert!(handle_weak.upgrade().is_none()); // note that upgrade fails in a thread
        ///     # return; // don't upgrade_in_event_loop in our examples
        ///     // now forward the data to the main thread using upgrade_in_event_loop
        ///     handle_weak.upgrade_in_event_loop(move |handle| handle.set_foo(foo));
        /// });
        /// # thread.join().unwrap(); return; // don't run the event loop in examples
        /// handle.run();
        /// ```
        #[cfg(feature = "std")]
        pub fn upgrade_in_event_loop(
            &self,
            func: impl FnOnce(T) + Send + 'static,
        ) -> Result<(), EventLoopError>
        where
            T: 'static,
        {
            let weak_handle = self.clone();
            super::invoke_from_event_loop(move || {
                if let Some(h) = weak_handle.upgrade() {
                    func(h);
                }
            })
        }
    }

    // Safety: we make sure in upgrade that the thread is the proper one,
    // and the VWeak only use atomic pointer so it is safe to clone and drop in another thread
    #[allow(unsafe_code)]
    #[cfg(feature = "std")]
    unsafe impl<T: ComponentHandle> Send for Weak<T> {}
}

pub use weak_handle::*;

/// Adds the specified function to an internal queue, notifies the event loop to wake up.
/// Once woken up, any queued up functors will be invoked.
///
/// This function is thread-safe and can be called from any thread, including the one
/// running the event loop. The provided functors will only be invoked from the thread
/// that started the event loop.
///
/// You can use this to set properties or use any other Slint APIs from other threads,
/// by collecting the code in a functor and queuing it up for invocation within the event loop.
///
/// See also [`Weak::upgrade_in_event_loop`]
///
/// # Example
/// ```rust
/// slint::slint! { MyApp := Window { property <int> foo; /* ... */ } }
/// # i_slint_backend_testing::init();
/// let handle = MyApp::new();
/// let handle_weak = handle.as_weak();
/// # return; // don't run the event loop in examples
/// let thread = std::thread::spawn(move || {
///     // ... Do some computation in the thread
///     let foo = 42;
///      // now forward the data to the main thread using invoke_from_event_loop
///     let handle_copy = handle_weak.clone();
///     slint::invoke_from_event_loop(move || handle_copy.unwrap().set_foo(foo));
/// });
/// handle.run();
/// ```
pub fn invoke_from_event_loop(func: impl FnOnce() + Send + 'static) -> Result<(), EventLoopError> {
    crate::platform::event_loop_proxy()
        .ok_or(EventLoopError::NoEventLoopProvider)?
        .invoke_from_event_loop(alloc::boxed::Box::new(func))
}

/// Schedules the main event loop for termination. This function is meant
/// to be called from callbacks triggered by the UI. After calling the function,
/// it will return immediately and once control is passed back to the event loop,
/// the initial call to `slint::run_event_loop()` will return.
pub fn quit_event_loop() -> Result<(), EventLoopError> {
    crate::platform::event_loop_proxy()
        .ok_or(EventLoopError::NoEventLoopProvider)?
        .quit_event_loop()
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
/// Error returned from the [`invoke_from_event_loop()`] and [`quit_event_loop()`] function
pub enum EventLoopError {
    /// The event could not be sent because the event loop was terminated already
    EventLoopTerminated,
    /// The event could not be sent because the Slint platform abstraction was not yet initialized,
    /// or the platform does not support event loop.
    NoEventLoopProvider,
}

#[test]
fn logical_physical_pos() {
    use crate::graphics::euclid::approxeq::ApproxEq;

    let phys = PhysicalPosition::new(100, 50);
    let logical = phys.to_logical(2.);
    assert!(logical.x.approx_eq(&50.));
    assert!(logical.y.approx_eq(&25.));

    assert_eq!(logical.to_physical(2.), phys);
}

#[test]
fn logical_physical_size() {
    use crate::graphics::euclid::approxeq::ApproxEq;

    let phys = PhysicalSize::new(100, 50);
    let logical = phys.to_logical(2.);
    assert!(logical.width.approx_eq(&50.));
    assert!(logical.height.approx_eq(&25.));

    assert_eq!(logical.to_physical(2.), phys);
}
