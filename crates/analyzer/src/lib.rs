use std::path::PathBuf;

use biome_js_syntax::{AnyJsRoot, JsSyntaxNode};
use rustc_hash::FxHashMap;
use type_info::type_info::TypeInfo;

#[derive(Default)]
pub struct Analyzer {
    type_info_table: FxHashMap<PathBuf, FxHashMap<JsSyntaxNode, TypeInfo>>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            type_info_table: FxHashMap::default(),
        }
    }

    pub fn get_type_info(&self, path: &PathBuf, node: &JsSyntaxNode) -> Option<TypeInfo> {
        self.type_info_table
            .get(path)
            .and_then(|file_cache| file_cache.get(node))
            .cloned()
    }

    pub fn analyze(&mut self, path: &PathBuf, ast: AnyJsRoot) {
        todo!();
    }
}
