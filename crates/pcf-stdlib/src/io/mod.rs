use crate::StandardModule;

#[derive(Debug, Default)]
pub struct IoModule;

impl StandardModule for IoModule {
    fn name(&self) -> &'static str {
        "std.io"
    }
}
