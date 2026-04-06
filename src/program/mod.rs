use std::collections::HashMap;
use crate::ast::{Node, NodeId};

pub struct Program {
    pub nodes: HashMap<NodeId, Node>,
    pub source_files: HashMap<String, SourceFile>,
}

pub struct SourceFile {
    pub path: String,
    pub content: String,
    pub ast: NodeId,
}
