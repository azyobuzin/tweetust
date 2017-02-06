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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadInitCommandResponse {
    pub expires_after_secs: u32,
    pub media_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessingInfo {
    pub state: String,
    pub check_after_secs: Option<u32>,
    pub progress_percent: Option<u8>,
    pub error: Option<MediaProcessingError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaProcessingError {
    pub code: i32,
    pub name: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadFinalizeCommandResponse {
    pub expires_after_secs: u32,
    pub image: Option<UploadedImage>,
    pub media_id: i64,
    pub processing_info: Option<ProcessingInfo>,
    pub size: u64,
    pub video: Option<UploadedVideo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadStatusCommandResponse {
    pub media_id: i64,
    pub processing_info: ProcessingInfo,
}
