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
