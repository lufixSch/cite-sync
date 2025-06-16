use poem_openapi::Tags;

/// OpenAPI Category Tags for API endpoints
#[derive(Tags)]
pub enum CategoryTags {
    /// General CiteSync API routes
    CiteSync,

    /// OPDS Server routes
    Opds,
}
