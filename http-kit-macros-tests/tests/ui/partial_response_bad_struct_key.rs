use http_kit_macros::PartialResponse;

#[derive(PartialResponse)]
#[partial_response(bogus = 1)]
struct X {
    pub id: i32,
}

fn main() {}
