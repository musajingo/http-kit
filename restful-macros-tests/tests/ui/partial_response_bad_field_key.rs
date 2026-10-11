use restful_macros::PartialResponse;

#[derive(PartialResponse)]
struct X {
    #[partial_response(bogus = 1)]
    pub id: i32,
}

fn main() {}
