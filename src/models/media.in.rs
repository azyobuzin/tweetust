#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaUploadResponse {
    pub expires_after_secs: u32,
    pub image: Option<UploadedImage>,
    pub media_id: i64,
    pub size: u64,
    pub video: Option<UploadedVideo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadedImage {
    pub w: u32,
    pub h: u32,
    pub image_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadedVideo {
    pub video_type: String,
}
