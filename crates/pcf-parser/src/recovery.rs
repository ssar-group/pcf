/// Tracks whether the parser is currently recovering from an error.
#[derive(Debug, Clone, Default)]
pub struct RecoveryState {
    active: bool,
}

impl RecoveryState {
    /// Starts a recovery block.
    pub fn begin(&mut self) {
        self.active = true;
    }

    /// Ends a recovery block.
    pub fn end(&mut self) {
        self.active = false;
    }

    /// Returns whether the parser is currently in recovery mode.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Resets the internal state without requiring a beginning/end pair.
    pub fn reset(&mut self) {
        self.active = false;
    }
}
