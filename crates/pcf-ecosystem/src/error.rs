#[derive(Debug, thiserror::Error)]
pub enum EcosystemError {
    #[error("ecosystem error")]
    Generic,
}
