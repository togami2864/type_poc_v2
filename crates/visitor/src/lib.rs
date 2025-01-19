use biome_rowan::Language;
use biome_rowan::{SyntaxNode, WalkEvent};

pub trait AstVisitor {
    type Language: Language;

    fn visit(&mut self, event: WalkEvent<SyntaxNode<Self::Language>>);
}
