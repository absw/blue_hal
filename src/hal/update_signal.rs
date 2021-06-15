/// Indicates the state of an update signal.
pub enum UpdateSignalResult {
    /// Do not update.
    None,

    /// Allow updates, if one is available.
    Any,

    /// Update from a specific image.
    Index(u32), // TODO: Use proper type for image bank.
}

pub trait UpdateSignal {
    fn should_update(&self) -> UpdateSignalResult;
}
