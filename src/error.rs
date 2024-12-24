/// Errors that can occur during Baserow authentication
///
/// These errors represent various authentication failures that may occur
/// when trying to authenticate with a Baserow instance.
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
    #[error("Missing {0} credentials")]
    MissingCredentials(&'static str),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
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
