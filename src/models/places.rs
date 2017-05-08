#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    pub attributes: HashMap<String, String>,
    pub bounding_box: Option<Box<BoundingBox>>,
    pub centroid: Option<Coordinates>,
    pub country: String,
    pub country_code: String,
    pub contained_within: Option<Vec<Place>>,
    pub full_name: String,
    pub geometry: Option<Box<Geometry>>,
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
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tup = try!(serializer.serialize_tuple(2));
        try!(tup.serialize_element(&self.longitude));
        try!(tup.serialize_element(&self.latitude));
        tup.end()
    }
}

impl<'x> Deserialize<'x> for Coordinates {
    fn deserialize<D: Deserializer<'x>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'x> de::Visitor<'x> for Visitor {
            type Value = Coordinates;

            fn visit_seq<A: de::SeqAccess<'x>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let longitude: f64 = try!(access.next_element().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(0, &self))));
                let latitude: f64 = try!(access.next_element().and_then(|x| x.ok_or_else(|| de::Error::invalid_length(1, &self))));
                Ok(Coordinates { longitude: longitude, latitude: latitude })
            }

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "an array of two integers")
            }
        }

        deserializer.deserialize_tuple(2, Visitor)
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
    pub as_of: chrono::DateTime<chrono::UTC>,
    pub created_at: chrono::DateTime<chrono::UTC>,
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
