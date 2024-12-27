use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnails {
    pub tiny: Thumbnail,
    pub small: Thumbnail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub url: String,
    pub thumbnails: Option<Thumbnails>,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub is_image: bool,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub uploaded_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_file_serialization() {
        // Test with all fields present
        let file_with_dimensions = File {
            url: "https://example.com/file.jpg".to_string(),
            thumbnails: Some(Thumbnails {
                tiny: Thumbnail {
                    url: "https://example.com/tiny.jpg".to_string(),
                    width: None,
                    height: Some(100),
                },
                small: Thumbnail {
                    url: "https://example.com/small.jpg".to_string(),
                    width: Some(200),
                    height: Some(200),
                },
            }),
            name: "file.jpg".to_string(),
            size: 1024,
            mime_type: "image/jpeg".to_string(),
            is_image: true,
            image_width: Some(800),
            image_height: Some(600),
            uploaded_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let json_with_dimensions = serde_json::to_value(&file_with_dimensions).unwrap();
        assert_eq!(json_with_dimensions["image_width"], 800);
        assert_eq!(json_with_dimensions["image_height"], 600);
        assert!(json_with_dimensions["thumbnails"].is_object());

        // Test with null dimensions and thumbnails
        let file_without_dimensions = File {
            url: "https://example.com/file.txt".to_string(),
            thumbnails: None,
            name: "file.txt".to_string(),
            size: 1024,
            mime_type: "text/plain".to_string(),
            is_image: false,
            image_width: None,
            image_height: None,
            uploaded_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let json_without_dimensions = serde_json::to_value(&file_without_dimensions).unwrap();
        assert!(json_without_dimensions["image_width"].is_null());
        assert!(json_without_dimensions["image_height"].is_null());
        assert!(json_without_dimensions["thumbnails"].is_null());

        // Test deserialization from JSON
        let json = json!({
            "url": "https://example.com/file.txt",
            "thumbnails": null,
            "name": "file.txt",
            "size": 1024,
            "mime_type": "text/plain",
            "is_image": false,
            "image_width": null,
            "image_height": null,
            "uploaded_at": "2023-01-01T00:00:00Z"
        });

        let file: File = serde_json::from_value(json).unwrap();
        assert!(file.image_width.is_none());
        assert!(file.image_height.is_none());
        assert!(file.thumbnails.is_none());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadFileViaUrlRequest {
    pub url: String,
}
