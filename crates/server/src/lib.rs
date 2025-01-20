use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use analyzer::Analyzer;
use biome_js_parser::{parse, JsParserOptions};
use biome_js_syntax::{AnyJsRoot, JsFileSource, JsSyntaxNode};
use type_info::type_info::TypeInfo;
use visitor::AstVisitor;

#[derive(Default)]
pub struct Server {
    analyzer: Arc<Mutex<Analyzer>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(Analyzer::new())),
        }
    }

    pub fn init(&self, user_paths: Vec<PathBuf>) -> Result<(), std::io::Error> {
        let mut analyzer = self.analyzer.lock().unwrap();

        for path in user_paths {
            let content = fs::read_to_string(&path)?;
            let ast = self.parse(&content);
            analyzer.set_current_path(path);
            analyzer.visit(&ast);
        }
        Ok(())
    }

    pub fn parse(&self, source: &str) -> AnyJsRoot {
        let source_type = JsFileSource::ts();
        parse(source, source_type, JsParserOptions::default()).tree()
    }

    pub fn get_type_info(&self, path: &PathBuf, node: &JsSyntaxNode) -> Option<TypeInfo> {
        let analyzer = self.analyzer.lock().unwrap();
        analyzer.get_type_info(path, node)
    }
}
