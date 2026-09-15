use crate::StandardModule;

#[derive(Debug, Default)]
pub struct EnvModule;

impl StandardModule for EnvModule {
    fn name(&self) -> &'static str {
        "std.env"
    }
}
