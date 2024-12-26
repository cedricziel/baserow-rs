use std::{error::Error, fs::File};

use reqwest::{Client, Request, Response};
use tracing::{debug, error, info, instrument, trace, warn, Instrument, span, Level};

use crate::{
    api::file::File as BaserowFile,
    error::{FileUploadError, TokenAuthError},
    BaserowTable, Configuration, TableField,
};

#[async_trait::async_trait]
pub(crate) trait RequestTracing {
    /// Trace an HTTP request and its response
    #[instrument(skip(self, client, request), fields(method = %request.method(), url = %request.url()), err)]
    async fn trace_request(&self, client: &Client, request: Request) -> reqwest::Result<Response> {
        let span = span!(
            Level::DEBUG,
            "http_request",
            method = %request.method(),
            url = %request.url(),
        );

        async move {
            debug!("Sending HTTP request");
            trace!(headers = ?request.headers(), "Request headers");
            let response = client.execute(request).await?;
            let status = response.status();

            if status.is_success() {
                info!(status = %status, "HTTP request successful");
                trace!(headers = ?response.headers(), "Response headers");
            } else {
                error!(status = %status, "HTTP request failed");
                warn!(headers = ?response.headers(), "Failed response headers");
            }

            Ok(response)
        }
        .instrument(span)
        .await
    }
}

impl<T: BaserowClient + ?Sized> RequestTracing for T {}

/// Trait defining the public API interface for Baserow
///
/// This trait includes tracing for all operations, providing detailed logs
/// about HTTP requests, responses, and any errors that occur.
///
/// All HTTP operations are automatically traced through the RequestTracing trait,
/// which provides detailed logging of request/response cycles.
#[async_trait::async_trait]
pub trait BaserowClient: RequestTracing {
    /// Authenticates an existing user based on their email and their password.
    /// If successful, an access token and a refresh token will be returned.
    ///
    /// This operation is traced with detailed logging of the authentication process,
    /// excluding sensitive information like credentials.
    async fn token_auth(&self) -> Result<Box<dyn BaserowClient>, TokenAuthError>;

    /// Retrieves all fields for a given table.
    ///
    /// This operation is traced with detailed logging of the request/response cycle
    /// and field retrieval results.
    async fn table_fields(&self, table_id: u64) -> Result<Vec<TableField>, Box<dyn Error>>;

    /// Returns a table by its ID.
    fn table_by_id(&self, id: u64) -> BaserowTable;

    /// Upload a file to Baserow
    ///
    /// This operation is traced with detailed logging of the upload process,
    /// including file metadata and upload status.
    async fn upload_file(
        &self,
        file: File,
        filename: String,
    ) -> Result<BaserowFile, FileUploadError>;

    /// Upload a file to Baserow via URL
    ///
    /// This operation is traced with detailed logging of the URL validation
    /// and upload process.
    async fn upload_file_via_url(&self, url: &str) -> Result<BaserowFile, FileUploadError>;

    /// Get the underlying configuration
    fn get_configuration(&self) -> Configuration;

    /// Get the underlying HTTP client
    fn get_client(&self) -> Client;
}
