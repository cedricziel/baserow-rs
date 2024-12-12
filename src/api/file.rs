use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnails {
    pub tiny: Thumbnail,
    pub small: Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub url: String,
    pub thumbnails: Thumbnails,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub is_image: bool,
    pub image_width: u32,
    pub image_height: u32,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileViaUrlRequest {
    pub url: String,
}
