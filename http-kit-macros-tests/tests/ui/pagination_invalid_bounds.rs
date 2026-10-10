#[http_kit_macros::cursor_pagination(default_limit = 0, skip(serde, validate, utoipa))]
struct ZeroCursorDefault {}

#[http_kit_macros::cursor_pagination(max_limit = 0, skip(serde, validate, utoipa))]
struct ZeroCursorMaximum {}

#[http_kit_macros::cursor_pagination(default_limit = 11, max_limit = 10, skip(serde, validate, utoipa))]
struct CursorDefaultAboveMaximum {}

#[http_kit_macros::offset_pagination(default_page_size = 0, skip(serde, validate, utoipa))]
struct ZeroOffsetDefault {}

#[http_kit_macros::offset_pagination(max_page_size = 0, skip(serde, validate, utoipa))]
struct ZeroOffsetMaximum {}

#[http_kit_macros::offset_pagination(default_page_size = 11, max_page_size = 10, skip(serde, validate, utoipa))]
struct OffsetDefaultAboveMaximum {}

fn main() {}
