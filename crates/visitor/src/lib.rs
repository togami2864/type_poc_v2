use biome_js_syntax::AnyJsRoot;

pub trait AstVisitor {
    fn visit(&mut self, ast: &AnyJsRoot);
}
