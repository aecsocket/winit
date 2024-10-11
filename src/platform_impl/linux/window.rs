use adw::prelude::*;

use crate::{
    error::{NotSupportedError, RequestError},
    window::WindowId,
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
    fn new(inner: adw::Window) -> Self {
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

    fn inner_position(&self) -> Result<dpi::PhysicalPosition<i32>, crate::error::RequestError> {
        Err(NotSupportedError::new("inner_position is not supported").into()) // TODO
    }
}
