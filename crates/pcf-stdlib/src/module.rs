use pcf_runtime::RuntimeRegistry;

#[derive(Debug, thiserror::Error)]
pub enum StdlibError {
    #[error("stdlib error")]
    Generic,
}

pub trait StandardModule {
    fn name(&self) -> &'static str;

    fn register(&self, _registry: &mut RuntimeRegistry) -> Result<(), StdlibError> {
        Ok(())
    }
}
