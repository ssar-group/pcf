use crate::StandardModule;

#[derive(Debug, Default)]
pub struct JsonModule;

impl StandardModule for JsonModule {
    fn name(&self) -> &'static str {
        "std.json"
    }
}
