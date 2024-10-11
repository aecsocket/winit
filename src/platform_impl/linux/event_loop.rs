use std::cell::Cell;

use adw::prelude::*;

use crate::{
    application::ApplicationHandler,
    error::EventLoopError,
    event_loop::ControlFlow,
    window::{Theme, WindowAttributes},
};

use super::{display_handle_from_gdk, MonitorHandle, OwnedDisplayHandle, Window};

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
            .ok_or_else(|| os_error!("failed to get default Wayland display"))?;

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
        window_attributes: WindowAttributes,
    ) -> Result<Box<dyn crate::window::Window>, crate::error::RequestError> {
        Ok(Box::new(Window::new(window_attributes)))
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
            adw::ColorScheme::Default => None,
            adw::ColorScheme::PreferLight | adw::ColorScheme::ForceLight => Some(Theme::Light),
            adw::ColorScheme::PreferDark | adw::ColorScheme::ForceDark => Some(Theme::Dark),
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
