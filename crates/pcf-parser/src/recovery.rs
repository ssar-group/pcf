#[derive(Debug, Clone, Default)]
pub struct RecoveryState {
    pub active: bool,
}

impl RecoveryState {
    pub fn begin(&mut self) {
        self.active = true;
    }

    pub fn end(&mut self) {
        self.active = false;
    }
}
