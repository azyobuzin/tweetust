#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    pub attributes: HashMap<String, String>,
    pub bounding_box: BoundingBox,
    pub centroid: Option<Coordinates>,
    pub country: String,
    pub country_code: String,
    pub contained_within: Option<Vec<Place>>,
    pub full_name: String,
    pub geometry: Option<Geometry>,
    pub id: String,
    pub name: String,
    pub place_type: String,
    //pub polylines: Option<Vec<?>>,
    pub url: String,
    pub vendor_info: Option<GeoVendorInfo>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub coordinates: Vec<Vec<Coordinates>>,
    #[serde(rename = "type")]
    pub box_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: Coordinates,
}

#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub longitude: f64,
    pub latitude: f64,
}

impl Serialize for Coordinates {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        let mut state = try!(serializer.serialize_seq(Some(2)));
        try!(serializer.serialize_seq_elt(&mut state, self.longitude));
        try!(serializer.serialize_seq_elt(&mut state, self.latitude));
        serializer.serialize_seq_end(state)
    }
}

impl Deserialize for Coordinates {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = Coordinates;
            fn visit_seq<V: de::SeqVisitor>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error> {
                let longitude: f64 = try!(visitor.visit().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(0))));
                let latitude: f64 = try!(visitor.visit().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(1))));
                try!(visitor.end());
                Ok(Coordinates { longitude: longitude, latitude: latitude })
            }
        }

        deserializer.deserialize_seq_fixed_size(2, Visitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoVendorInfo {
    pub yelp: Option<Box<VendorYelp>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorYelp {
    pub business_id: String,
    pub mobile_url: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendPlace {
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
    pub name: String,
    pub parentid: i64,
    #[serde(rename = "placeType")]
    pub place_type: PlaceType,
    pub url: String,
    pub woeid: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaceType {
    pub code: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendsResult {
    // TODO: as_of と created_at はほかの日時形式と違う
    pub as_of: String,
    pub created_at: String,
    pub locations: Vec<TrendLocation>,
    pub trends: Vec<Trend>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendLocation {
    pub name: String,
    pub woeid: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trend {
    pub name: String,
    pub url: String,
    // pub promoted_content: Option<?>,
    pub query: String,
    pub tweet_volume: Option<u32>,
}
