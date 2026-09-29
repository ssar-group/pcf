use pcf_span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
}

impl HttpMethod {
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "PATCH" => Self::Patch,
            "DELETE" => Self::Delete,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            "CONNECT" => Self::Connect,
            "TRACE" => Self::Trace,
            _ => return None,
        })
    }
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Trace => "TRACE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDeclaration {
    pub methods: Vec<(HttpMethod, Span)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDeclaration {
    pub path: String,
    pub path_span: Span,
    pub method: HttpMethod,
    pub method_span: Span,
    pub body: Vec<crate::Statement>,
    pub span: Span,
}
