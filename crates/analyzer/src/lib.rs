use biome_js_syntax::{
    AnyJsExpression, AnyJsLiteralExpression, AnyJsModuleItem, AnyJsRoot, AnyJsStatement,
    JsExpressionStatement, JsModule, JsSyntaxNode,
};
use biome_rowan::AstNode;
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use type_info::type_info::TypeInfo;
use visitor::AstVisitor;

#[derive(Default)]
pub struct Analyzer {
    current_path: PathBuf,
    type_info_table: FxHashMap<PathBuf, FxHashMap<JsSyntaxNode, TypeInfo>>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::new(),
            type_info_table: FxHashMap::default(),
        }
    }

    pub fn get_type_info(&self, path: &PathBuf, node: &JsSyntaxNode) -> Option<TypeInfo> {
        self.type_info_table
            .get(path)
            .and_then(|file_cache| file_cache.get(node))
            .cloned()
    }

    pub fn set_current_path(&mut self, path: PathBuf) {
        self.current_path = path;
    }
}

impl AstVisitor for Analyzer {
    fn visit(&mut self, ast: &AnyJsRoot) {
        self.visit_node(ast);
    }
}

impl Analyzer {
    fn visit_node(&mut self, node: &AnyJsRoot) {
        match node {
            AnyJsRoot::JsModule(node) => {
                self.visit_module(node);
            }
            _ => todo!(),
        }
    }

    fn visit_module(&mut self, node: &JsModule) {
        for item in node.items() {
            match item {
                AnyJsModuleItem::AnyJsStatement(node) => {
                    self.visit_statement(&node);
                }
                _ => todo!(),
            }
        }
    }

    fn visit_statement(&mut self, node: &AnyJsStatement) {
        match node {
            AnyJsStatement::JsExpressionStatement(node) => {
                self.visit_expression_statement(node);
            }
            _ => todo!(),
        }
    }

    fn visit_expression_statement(&mut self, node: &JsExpressionStatement) {
        let expr = node.expression().unwrap();
        self.visit_expression(&expr);
    }

    fn visit_expression(&mut self, node: &AnyJsExpression) {
        match node {
            AnyJsExpression::AnyJsLiteralExpression(node) => match node {
                AnyJsLiteralExpression::JsNumberLiteralExpression(node) => {
                    let value = node.syntax().text().to_string();
                    self.type_info_table
                        .entry(self.current_path.clone())
                        .or_default()
                        .insert(node.clone().into(), TypeInfo::Literal { value });
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
