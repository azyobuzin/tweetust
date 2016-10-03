#[derive(Clone, Debug, Serialize, Deserialize)]
struct InternalErrorResponse {
    errors: Option<Vec<Error>>,
    error: Option<Vec<Error>>
}
