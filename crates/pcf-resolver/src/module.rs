#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    pub segments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedModule {
    pub path: ModulePath,
}
