#[derive(Clone, Show, RustcDecodable)]
pub struct Place {
    //pub attributes: BTreeMap<String, json::Json>,
    pub bounding_box: BoundingBox,
    pub country: String,
    pub country_code: String,
    pub full_name: String,
    pub id: String,
    pub name: String,
    pub place_type: String,
    pub url: String
}

#[derive(Clone, Show, RustcDecodable)]
pub struct BoundingBox {
    pub coordinates: Vec<Vec<Vec<f64>>>,
    //pub type: String
}
