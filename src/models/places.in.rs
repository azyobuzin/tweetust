pub mod places {
    use std::collections::BTreeMap;

    #[derive(Clone, Debug, Serialize, Deserialize)]
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

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BoundingBox {
        pub coordinates: Vec<Vec<Vec<f64>>>,
        #[serde(rename = "type")]
        pub box_type: String
    }
}
