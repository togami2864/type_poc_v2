use std::path::PathBuf;

use biome_js_syntax::JsSyntaxNode;
use rustc_hash::FxHashMap;
use type_info::type_info::TypeInfo;

pub struct Analyzer {
    type_info_table: FxHashMap<PathBuf, FxHashMap<JsSyntaxNode, TypeInfo>>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            type_info_table: FxHashMap::default(),
        }
    }

    pub fn analyze(&mut self, path: &PathBuf, ast: JsSyntaxNode) {
        todo!()
    }
}
