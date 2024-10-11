use std::num::{NonZeroU16, NonZeroU32};

use adw::{gdk, prelude::*};
use dpi::{PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedDisplayHandle {
    pub(crate) inner: gdk::Display,
}

#[cfg(feature = "rwh_06")]
impl rwh_06::HasDisplayHandle for OwnedDisplayHandle {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        display_handle_from_gdk(&self.inner)
    }
}

#[cfg(feature = "rwh_06")]
pub(crate) fn display_handle_from_gdk(
    display: &gdk::Display,
) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
    use gdk_wayland::wayland_client::Proxy;

    if let Some(display) = display.downcast_ref::<gdk_wayland::WaylandDisplay>() {
        let display = display.wl_display().ok_or(rwh_06::HandleError::Unavailable)?;
        let display_ptr = std::ptr::NonNull::new(display.id().as_ptr() as *mut _)
            .expect("wl_display should not be null");

        let raw = rwh_06::WaylandDisplayHandle::new(display_ptr);
        // SAFETY: `display_ptr` should be a valid handle to a `wl_display`
        return Ok(unsafe { rwh_06::DisplayHandle::borrow_raw(raw.into()) });
    }

    if let Some(display) = display.downcast_ref::<gdk_x11::X11Display>() {
        // SAFETY: `gdk_x11` doesn't actually lay out any safety invariants here.
        // However, we're not doing anything with this pointer other than turning
        // it into a `c_void` and `NonNull`.
        // https://docs.gtk.org/gdk4-x11/method.X11Display.get_xdisplay.html
        let xdisplay = unsafe { display.xdisplay() } as *mut std::ffi::c_void;
        let xdisplay = std::ptr::NonNull::new(xdisplay);

        let screen = display.screen().screen_number();

        let raw = rwh_06::XlibDisplayHandle::new(xdisplay, screen);
        // SAFETY: GDK should give us a pointer to a valid X display
        return Ok(unsafe { rwh_06::DisplayHandle::borrow_raw(raw.into()) });
    }

    unreachable!("`GdkDisplay` should either be a `WaylandDisplay` or an `X11Display`");
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorHandle {
    pub(crate) inner: gdk::Monitor,
}

impl MonitorHandle {
    pub fn new(proxy: gdk::Monitor) -> Self {
        Self { inner: proxy }
    }

    pub fn name(&self) -> Option<String> {
        self.inner.description().map(String::from)
    }

    pub fn position(&self) -> Option<PhysicalPosition<i32>> {
        // https://docs.gtk.org/gdk3/method.Monitor.get_geometry.html
        // `geometry` is in "application" (logical) pixels,
        // so the actual position is `geometry * scale_factor`
        let geometry = self.inner.geometry();
        let scale_factor = self.inner.scale_factor();
        let position =
            PhysicalPosition { x: geometry.x() * scale_factor, y: geometry.y() * scale_factor };

        Some(position)
    }

    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor() as f64
    }

    pub fn current_video_mode(&self) -> Option<VideoModeHandle> {
        None // TODO
    }

    pub fn video_modes(&self) -> impl Iterator<Item = VideoModeHandle> {
        std::iter::empty() // TODO
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoModeHandle {
    monitor: MonitorHandle,
    size: PhysicalSize<u32>,
    refresh_rate_millihertz: Option<NonZeroU32>,
}

impl VideoModeHandle {
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn bit_depth(&self) -> Option<NonZeroU16> {
        None // TODO: gdk::Visuals has some info on this?
    }

    pub fn refresh_rate_millihertz(&self) -> Option<NonZeroU32> {
        self.refresh_rate_millihertz
    }

    pub fn monitor(&self) -> MonitorHandle {
        self.monitor.clone()
    }
}
