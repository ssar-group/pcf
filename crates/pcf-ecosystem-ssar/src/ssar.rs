use pcf_ecosystem::Ecosystem;
use pcf_runtime::RuntimeRegistry;

#[derive(Debug, Default)]
pub struct SsarEcosystem;

impl Ecosystem for SsarEcosystem {
    fn id(&self) -> &str {
        "ssar"
    }

    fn name(&self) -> &str {
        "SSAR"
    }

    fn register(&self, _registry: &mut RuntimeRegistry) -> Result<(), pcf_ecosystem::EcosystemError> {
        Ok(())
    }
}
