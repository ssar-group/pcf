use crate::{Environment, PermissionSet, RuntimeRegistry};

#[derive(Debug, Default)]
pub struct Runtime {
    pub registry: RuntimeRegistry,
    pub environment: Environment,
    pub permissions: PermissionSet,
}
