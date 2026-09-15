use crate::StandardModule;

#[derive(Debug, Default)]
pub struct TimeModule;

impl StandardModule for TimeModule {
    fn name(&self) -> &'static str {
        "std.time"
    }
}
