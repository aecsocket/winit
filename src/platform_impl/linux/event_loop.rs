use std::cell::Cell;

use adw::{gdk, glib, prelude::*, ColorScheme};
use dpi::LogicalSize;

use crate::{
    application::ApplicationHandler,
    error::EventLoopError,
    event_loop::ControlFlow,
    window::{Fullscreen, Theme},
};

use super::{display_handle_from_gdk, output::MonitorHandle, OwnedDisplayHandle};

#[derive(Debug)]
pub struct EventLoop {
    main_loop: glib::MainLoop,
    active_event_loop: ActiveEventLoop,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PlatformSpecificEventLoopAttributes;

impl EventLoop {
    pub fn new(_attributes: &PlatformSpecificEventLoopAttributes) -> Result<Self, EventLoopError> {
        adw::init()
            .map_err(|err| os_error!(format!("failed to initialize `libadwaita`: {err}")))?;
        let display = gdk::Display::default()
            .ok_or_else(|| os_error!("failed to get default `libadwaita` Wayland display"))?;

        let main_context = glib::MainContext::default();
        Ok(Self {
            main_loop: glib::MainLoop::new(
                Some(&main_context),
                false, // is_running
            ),
            active_event_loop: ActiveEventLoop {
                main_context,
                display,
                control_flow: Cell::new(ControlFlow::default()),
                exit: Cell::new(None),
            },
        })
    }

    pub fn run_app<A: ApplicationHandler>(self, app: A) -> Result<(), EventLoopError> {
        // TODO
        self.main_loop.run();
        Ok(())
    }

    pub fn window_target(&self) -> &dyn crate::event_loop::ActiveEventLoop {
        &self.active_event_loop
    }
}

#[derive(Debug)]
pub struct ActiveEventLoop {
    main_context: glib::MainContext,
    pub(crate) display: gdk::Display,
    control_flow: Cell<ControlFlow>,
    exit: Cell<Option<i32>>,
}

impl crate::event_loop::ActiveEventLoop for ActiveEventLoop {
    fn create_proxy(&self) -> crate::event_loop::EventLoopProxy {
        crate::event_loop::EventLoopProxy {
            event_loop_proxy: EventLoopProxy { main_context: self.main_context.clone() },
        }
    }

    fn create_window(
        &self,
        window_attributes: crate::window::WindowAttributes,
    ) -> Result<Box<dyn crate::window::Window>, crate::error::RequestError> {
        let builder = adw::Window::builder()
            // disable F10 opening the app menu,
            // since we don't even have an app menu
            .handle_menubar_accel(false)
            .resizable(window_attributes.resizable)
            .title(window_attributes.title)
            .maximized(window_attributes.maximized)
            .visible(window_attributes.visible)
            .decorated(window_attributes.decorations);

        let builder = if let Some(surface_size) = window_attributes.surface_size {
            // `width`, `height` are accepted as application (logical) units
            // so scale factor is 1
            // TODO i32 handling
            let LogicalSize { width, height } = surface_size.to_logical::<i32>(1.0);
            builder.default_width(width).default_height(height)
        } else {
            builder
        };

        let builder = if let Some(min_surface_size) = window_attributes.min_surface_size {
            // see above
            // TODO i32 handling
            let LogicalSize { width, height } = min_surface_size.to_logical::<i32>(1.0);
            builder.width_request(width).height_request(height)
        } else {
            builder
        };

        if let Some(preferred_theme) = window_attributes.preferred_theme {
            // TODO: do we want to force instead?
            let color_scheme = match preferred_theme {
                Theme::Light => adw::ColorScheme::PreferLight,
                Theme::Dark => adw::ColorScheme::PreferDark,
            };
            // TODO: this changes the style of *all* windows
            adw::StyleManager::default().set_color_scheme(color_scheme);
        }

        let window = builder.build();

        if let Some(fullscreen) = window_attributes.fullscreen {
            match fullscreen {
                Fullscreen::Exclusive(_) => { /* unsupported */ },
                Fullscreen::Borderless(Some(monitor)) => {
                    window.fullscreen_on_monitor(&monitor.inner.inner);
                },
                Fullscreen::Borderless(None) => {
                    window.fullscreen();
                },
            }
        }

        // TODO `platform_specific`

        // `max_surface_size` unsupported
        // `surface_resize_increments` unsupported
        // `position` unsupported - removed in GTK4, was X11 specific: <https://docs.gtk.org/gtk4/migrating-3to4.html>
        // `transparent` unsupported
        // `blur` unsupported
        // TODO `window_icon`
        // `content_protected` unsupported
        // `window_level` unsupported
        // `active` unsupported
        // TODO `cursor`
        // `parent_window` unsupported

        todo!()
    }

    fn create_custom_cursor(
        &self,
        custom_cursor: crate::cursor::CustomCursorSource,
    ) -> Result<crate::cursor::CustomCursor, crate::error::RequestError> {
        todo!()
    }

    fn available_monitors(&self) -> Box<dyn Iterator<Item = crate::monitor::MonitorHandle>> {
        let monitors = self
            .display
            .monitors()
            .into_iter()
            .map(|obj| {
                obj.expect("should not be mutating list during iteration")
                    .downcast::<gdk::Monitor>()
                    .map(MonitorHandle::new)
                    .map(|inner| crate::monitor::MonitorHandle { inner })
                    .expect("object should be a `gdk::Monitor`")
            })
            .collect::<Vec<_>>();
        Box::new(monitors.into_iter())
    }

    fn primary_monitor(&self) -> Option<crate::monitor::MonitorHandle> {
        None // unsupported
    }

    fn listen_device_events(&self, _allowed: crate::event_loop::DeviceEvents) {
        // unsupported
    }

    fn system_theme(&self) -> Option<Theme> {
        match adw::StyleManager::default().color_scheme() {
            ColorScheme::Default => None,
            ColorScheme::PreferLight | ColorScheme::ForceLight => Some(Theme::Light),
            ColorScheme::PreferDark | ColorScheme::ForceDark => Some(Theme::Dark),
            _ => None,
        }
    }

    fn set_control_flow(&self, control_flow: crate::event_loop::ControlFlow) {
        self.control_flow.set(control_flow);
    }

    fn control_flow(&self) -> ControlFlow {
        self.control_flow.get()
    }

    fn exit(&self) {
        self.exit.set(Some(0));
    }

    fn exiting(&self) -> bool {
        self.exit.get().is_some()
    }

    fn owned_display_handle(&self) -> crate::event_loop::OwnedDisplayHandle {
        crate::event_loop::OwnedDisplayHandle {
            platform: OwnedDisplayHandle { inner: self.display.clone() },
        }
    }

    #[cfg(feature = "rwh_06")]
    fn rwh_06_handle(&self) -> &dyn rwh_06::HasDisplayHandle {
        self
    }
}

#[derive(Debug)]
pub struct EventLoopProxy {
    main_context: glib::MainContext,
}

impl EventLoopProxy {
    pub fn wake_up(&self) {
        self.main_context.wakeup();
    }
}

#[cfg(feature = "rwh_06")]
impl rwh_06::HasDisplayHandle for ActiveEventLoop {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        display_handle_from_gdk(&self.display)
    }
}
