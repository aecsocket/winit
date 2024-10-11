use adw::prelude::*;
use dpi::{LogicalSize, PhysicalPosition, Position};

use crate::{
    error::{NotSupportedError, RequestError},
    window::{Fullscreen, Theme, WindowAttributes, WindowId},
};

pub struct Window {
    inner: adw::Window,
    window_id: WindowId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSpecificWindowAttributes;

impl Default for PlatformSpecificWindowAttributes {
    fn default() -> Self {
        Self
    }
}

impl Window {
    pub fn new(attributes: WindowAttributes) -> Self {
        let render_target = gtk::Picture::new();
        let render_target_container = {
            let graphics_offload = gtk::GraphicsOffload::builder()
                .black_background(true)
                .child(&render_target)
                .hexpand(true)
                .vexpand(true)
                .build();

            // Use a trick to detect when the actual render target
            // is resized, and send this new frame size to the app.
            // https://stackoverflow.com/questions/70488187/get-calculated-size-of-widget-in-gtk-4-0
            // +-----------------------+
            // |          WL           |  WL: width_listener  (height 0)
            // |-----------------------|  HL: height_listener (width 0)
            // |   |                   |
            // | H |      render       |
            // | L |      target       |
            // |   |                   |
            // +-----------------------+

            let width_listener = gtk::DrawingArea::builder().hexpand(true).build();
            width_listener.set_draw_func({
                let render_target_width = render_target_width.clone();
                move |_, _, width, _| {
                    render_target_width.store(width, Ordering::SeqCst);
                }
            });

            let height_listener = gtk::DrawingArea::builder().vexpand(true).build();
            height_listener.set_draw_func({
                let render_target_height = render_target_height.clone();
                move |_, _, _, height| {
                    render_target_height.store(height, Ordering::SeqCst);
                }
            });

            let frame_content_h = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            frame_content_h.append(&height_listener);
            frame_content_h.append(&graphics_offload);

            let frame_content_v = gtk::Box::new(gtk::Orientation::Vertical, 0);
            frame_content_v.append(&width_listener);
            frame_content_v.append(&frame_content_h);

            frame_content_v
        };

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let builder = adw::Window::builder()
            .content(&content)
            // disable F10 opening the app menu,
            // since we don't even have an app menu
            .handle_menubar_accel(false)
            .resizable(attributes.resizable)
            .title(attributes.title)
            .maximized(attributes.maximized)
            .visible(attributes.visible)
            .decorated(attributes.decorations);

        let builder = if let Some(surface_size) = attributes.surface_size {
            // `width`, `height` are accepted as application (logical) units
            // so scale factor is 1
            // TODO i32 handling
            let LogicalSize { width, height } = surface_size.to_logical::<i32>(1.0);
            builder.default_width(width).default_height(height)
        } else {
            builder
        };

        let builder = if let Some(min_surface_size) = attributes.min_surface_size {
            // see above
            // TODO i32 handling
            let LogicalSize { width, height } = min_surface_size.to_logical::<i32>(1.0);
            builder.width_request(width).height_request(height)
        } else {
            builder
        };

        if let Some(preferred_theme) = attributes.preferred_theme {
            // TODO: do we want to force instead?
            let color_scheme = match preferred_theme {
                Theme::Light => adw::ColorScheme::PreferLight,
                Theme::Dark => adw::ColorScheme::PreferDark,
            };
            // TODO: this changes the style of *all* windows
            adw::StyleManager::default().set_color_scheme(color_scheme);
        }

        let window = builder.build();

        if let Some(fullscreen) = attributes.fullscreen {
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

        let window_id = WindowId::from_raw(inner.as_ptr() as usize);

        Self { inner, window_id }
    }
}

impl crate::window::Window for Window {
    fn id(&self) -> WindowId {
        self.window_id
    }

    fn scale_factor(&self) -> f64 {
        self.inner.scale_factor() as f64
    }

    fn request_redraw(&self) {
        todo!()
    }

    fn pre_present_notify(&self) {
        todo!()
    }

    fn reset_dead_keys(&self) {
        todo!()
    }

    fn inner_position(&self) -> Result<PhysicalPosition<i32>, crate::error::RequestError> {
        Err(NotSupportedError::new("inner_position is not supported").into()) // TODO
    }

    fn outer_position(&self) -> Result<PhysicalPosition<i32>, RequestError> {
        Err(NotSupportedError::new("outer_position is not supported").into()) // TODO
    }

    fn set_outer_position(&self, position: Position) {
        // unsupported
    }

    fn surface_size(&self) -> dpi::PhysicalSize<u32> {
        self.inner.width()
    }
}
