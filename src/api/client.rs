use std::{error::Error, fs::File};

use reqwest::Client;

use crate::{
    api::file::File as BaserowFile,
    error::{FileUploadError, TokenAuthError},
    BaserowTable, Configuration, TableField,
};

/// Trait defining the public API interface for Baserow
#[async_trait::async_trait]
pub trait BaserowClient {
    /// Authenticates an existing user based on their email and their password.
    /// If successful, an access token and a refresh token will be returned.
    async fn token_auth(&self) -> Result<Box<dyn BaserowClient>, TokenAuthError>;

    /// Retrieves all fields for a given table.
    async fn table_fields(&self, table_id: u64) -> Result<Vec<TableField>, Box<dyn Error>>;

    /// Returns a table by its ID.
    fn table_by_id(&self, id: u64) -> BaserowTable;

    /// Upload a file to Baserow
    async fn upload_file(
        &self,
        file: File,
        filename: String,
    ) -> Result<BaserowFile, FileUploadError>;

    /// Upload a file to Baserow via URL
    async fn upload_file_via_url(&self, url: &str) -> Result<BaserowFile, FileUploadError>;

    /// Get the underlying configuration
    fn get_configuration(&self) -> Configuration;

    /// Get the underlying HTTP client
    fn get_client(&self) -> Client;
}
