use http_kit_macros::ProjectionField;

#[derive(ProjectionField)]
#[projection_field(name = "x", column = Foo)]
struct X {
    pub id: i32,
    #[projection_field(column = Foo::A)]
    #[column(Foo::B)]
    pub name: String,
}

fn main() {}
