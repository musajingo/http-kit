use restful_macros::ProjectionField;

#[derive(ProjectionField)]
#[projection_field(name = "x")]
struct X {
    pub id: i32,
    #[projection_skip]
    #[column(Foo::Bar)]
    pub secret: String,
}

fn main() {}
