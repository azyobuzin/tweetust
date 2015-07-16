use std::cmp;
use std::collections::BTreeMap;

#[derive(Clone, Debug, RustcDecodable)]
pub struct Place {
    pub attributes: BTreeMap<String, String>,
    pub bounding_box: BoundingBox,
    pub country: String,
    pub country_code: String,
    pub full_name: String,
    pub id: String,
    pub name: String,
    pub place_type: String,
    pub url: String
}

impl cmp::Eq for Place { }
impl cmp::PartialEq for Place {
    fn eq(&self, other: &Place) -> bool { self.id == other.id }
}

#[derive(Clone, Debug, RustcDecodable)]
pub struct BoundingBox {
    pub coordinates: Vec<Vec<Vec<f64>>>,
    pub type_: String
}
