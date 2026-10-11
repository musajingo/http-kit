use restful_macros::ProjectionField;

#[derive(ProjectionField)]
#[projection_field(name = "x")]
struct X {
    pub id: i32,
    #[projection_field(column)]
    pub name: String,
}

fn main() {}
