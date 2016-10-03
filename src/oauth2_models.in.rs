#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidateTokenResponse {
    pub access_token: String
}
