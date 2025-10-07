use std::collections::BTreeSet;

use egui_code_editor::Syntax;

pub fn wgsl_syntax() -> Syntax {
    Syntax {
        language: "Wgsl",
        case_sensitive: true,
        comment: "//",
        comment_multiline: ["/*", "*/"],
        hyperlinks: BTreeSet::from(["http"]),
        keywords: BTreeSet::from([]),
        types: BTreeSet::from(["i32", "f32", "f64", "i64"]),
        special: BTreeSet::from([]),
    }
}
