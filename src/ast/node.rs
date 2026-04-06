use super::token::Span;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Identifier,
    TypeDeclaration,
}

pub type NodeId = u32;
