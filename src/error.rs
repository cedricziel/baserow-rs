use tracing::{error, warn};

/// Errors that can occur during Baserow authentication
///
/// These errors represent various authentication failures that may occur
/// when trying to authenticate with a Baserow instance.
///
/// Each variant includes tracing to provide detailed context about the error.
#[derive(thiserror::Error, Debug)]
pub enum BaserowAuthenticationError {
    #[error("Authentication failed: Invalid credentials provided")]
    InvalidCredentials,
    #[error("Authentication failed: User account is deactivated")]
    DeactivatedUser,
    #[error("Authentication failed: Authentication provider is disabled")]
    AuthProviderDisabled,
    #[error("Authentication failed: Email verification is required")]
    EmailVerificationRequired,
}

impl BaserowAuthenticationError {
    pub(crate) fn log(&self) {
        match self {
            Self::InvalidCredentials => {
                warn!(error = %self, "Authentication attempt failed due to invalid credentials");
            }
            Self::DeactivatedUser => {
                warn!(error = %self, "Authentication attempt failed due to deactivated user account");
            }
            Self::AuthProviderDisabled => {
                error!(error = %self, "Authentication attempt failed due to disabled auth provider");
            }
            Self::EmailVerificationRequired => {
                warn!(error = %self, "Authentication requires email verification");
            }
        }
    }
}

/// Errors that can occur during token-based authentication
///
/// These errors represent failures that may occur when attempting to
/// authenticate using API tokens or JWT tokens.
///
/// # Example
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, error::TokenAuthError, api::client::BaserowClient};
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConfigBuilder::new()
///         .base_url("https://api.baserow.io")
///         .email("user@example.com")
///         .password("password")
///         .build();
///
///     let baserow = Baserow::with_configuration(config);
///     match baserow.token_auth().await {
///         Ok(client) => println!("Authentication successful"),
///         Err(TokenAuthError::MissingCredentials(field)) => {
///             println!("Missing required field: {}", field)
///         }
///         Err(e) => println!("Authentication failed: {}", e),
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum TokenAuthError {
    #[error("Authentication failed: Missing required {0} credentials")]
    MissingCredentials(&'static str),
    #[error("Token authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Network error during authentication: {0}")]
    NetworkError(#[from] reqwest::Error),
}

impl TokenAuthError {
    pub(crate) fn log(&self) {
        match self {
            Self::MissingCredentials(field) => {
                warn!(error = %self, field = %field, "Token authentication failed due to missing credentials");
            }
            Self::AuthenticationFailed(msg) => {
                error!(error = %self, details = %msg, "Token authentication failed");
            }
            Self::NetworkError(e) => {
                error!(error = %self, network_error = %e, "Token authentication failed due to network error");
            }
        }
    }
}

/// Errors that can occur during file uploads
///
/// These errors represent various failures that may occur when
/// uploading files to Baserow, either directly or via URL.
///
/// # Example
/// ```no_run
/// use baserow_rs::{ConfigBuilder, Baserow, error::FileUploadError, api::client::BaserowClient};
/// use std::fs::File;
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConfigBuilder::new()
///         .base_url("https://api.baserow.io")
///         .api_key("your-api-key")
///         .build();
///
///     let baserow = Baserow::with_configuration(config);
///     let file = File::open("image.jpg").unwrap();
///
///     match baserow.upload_file(file, "image.jpg".to_string()).await {
///         Ok(uploaded) => println!("File uploaded: {}", uploaded.url),
///         Err(FileUploadError::FileReadError(e)) => {
///             println!("Failed to read file: {}", e)
///         }
///         Err(e) => println!("Upload failed: {}", e),
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum FileUploadError {
    #[error("File upload failed: Unable to read file - {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("File upload failed: Network error - {0}")]
    UploadError(#[from] reqwest::Error),
    #[error("File upload failed: Invalid content type provided")]
    InvalidContentType,
    #[error("File upload failed: Server responded with unexpected status code {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
    #[error("File upload failed: Invalid URL provided - {0}")]
    InvalidURL(String),
}

impl FileUploadError {
    pub(crate) fn log(&self) {
        match self {
            Self::FileReadError(e) => {
                error!(error = %self, io_error = %e, "File upload failed due to file read error");
            }
            Self::UploadError(e) => {
                error!(error = %self, network_error = %e, "File upload failed due to network error");
            }
            Self::InvalidContentType => {
                warn!(error = %self, "File upload failed due to invalid content type");
            }
            Self::UnexpectedStatusCode(status) => {
                error!(error = %self, status_code = %status, "File upload failed with unexpected status code");
            }
            Self::InvalidURL(url) => {
                warn!(error = %self, url = %url, "File upload failed due to invalid URL");
            }
        }
    }
}
