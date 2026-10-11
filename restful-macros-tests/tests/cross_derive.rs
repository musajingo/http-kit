//! One struct deriving both `PartialResponse` (`row = …`) and `ProjectionField`
//! (`column = …`): the generated `From<PartialRow>` and `IntoColumnSelections`
//! impls must coexist.

use restful::projection::IntoColumnSelections;
use restful_macros::{PartialResponse, ProjectionField};
use serde::Serialize;

struct GizmoTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::EnumIter)]
enum GizmoColumn {
    Id,
    Name,
}

impl From<GizmoColumn> for &'static str {
    fn from(c: GizmoColumn) -> Self {
        match c {
            GizmoColumn::Id => "id",
            GizmoColumn::Name => "name",
        }
    }
}

impl pgkit::types::DatabaseTableColumn for GizmoColumn {
    type Table = GizmoTable;
    fn table_primary_key() -> Self {
        GizmoColumn::Id
    }
}

impl pgkit::types::DatabaseTable for GizmoTable {
    type Columns = GizmoColumn;
    const NAME: &'static str = "gizmo";
    fn primary_key() -> Self::Columns {
        GizmoColumn::Id
    }
}

#[allow(dead_code)]
struct GizmoRow {
    pub id: Option<u32>,
    pub name: Option<String>,
}

#[derive(Serialize, PartialResponse, ProjectionField)]
#[partial_response(row = GizmoRow)]
#[projection_field(name = "gizmo", column = GizmoColumn)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Gizmo {
    pub id: u32,
    pub name: String,
}

#[test]
fn both_derives_emit_their_impls() {
    // ProjectionField → IntoColumnSelections
    let sel = GizmoField::into_column_selections(vec![GizmoField::Id, GizmoField::Name]);
    assert!(sel.contains(&GizmoColumn::Id));
    assert!(sel.contains(&GizmoColumn::Name));

    // PartialResponse → From<GizmoRow> for PartialGizmo
    let p = PartialGizmo::from(GizmoRow {
        id: Some(1),
        name: Some("g".to_string()),
    });
    assert_eq!(p.id, Some(1));
    assert_eq!(p.name, Some("g".to_string()));
}
