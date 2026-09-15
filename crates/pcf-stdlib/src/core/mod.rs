use crate::StandardModule;

#[derive(Debug, Default)]
pub struct CoreModule;

impl StandardModule for CoreModule {
    fn name(&self) -> &'static str {
        "std.core"
    }
}
