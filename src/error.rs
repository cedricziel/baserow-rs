#[derive(thiserror::Error, Debug)]
pub enum BaserowAuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User is not active")]
    DeactivatedUser,
    #[error("Auth provider is disabled")]
    AuthProviderDisabled,
    #[error("Email verification is required")]
    EmailVerificationRequired,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenAuthError {
    #[error("Missing {0} credentials")]
    MissingCredentials(&'static str),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum FileUploadError {
    #[error("File read error: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Upload failed: {0}")]
    UploadError(#[from] reqwest::Error),
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Unexpected status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
    #[error("Invalid URL {0}")]
    InvalidURL(String),
}
