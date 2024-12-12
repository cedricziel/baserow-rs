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
