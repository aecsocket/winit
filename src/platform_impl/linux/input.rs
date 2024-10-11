// there is `gdk::Device`, a refcounted handle to a device, but it's non-`Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FingerId(usize);

impl FingerId {
    #[cfg(test)]
    pub const fn dummy() -> FingerId {
        FingerId(0)
    }
}
