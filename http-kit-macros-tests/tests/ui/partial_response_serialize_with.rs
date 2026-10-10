fn as_str<S: serde::Serializer>(v: &u32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}

#[derive(http_kit_macros::PartialResponse, serde::Serialize)]
struct Payload {
    #[serde(serialize_with = "as_str")]
    count: u32,
}

fn main() {}
