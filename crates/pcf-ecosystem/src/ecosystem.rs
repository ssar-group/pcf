use pcf_runtime::RuntimeRegistry;

use crate::EcosystemError;

pub trait Ecosystem {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    fn register(&self, _registry: &mut RuntimeRegistry) -> Result<(), EcosystemError> {
        Ok(())
    }
}
