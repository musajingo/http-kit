use http_kit_macros::PartialResponse;

#[derive(PartialResponse)]
struct X {
    #[partial_response(skip, row_field = other)]
    pub id: i32,
}

fn main() {}
