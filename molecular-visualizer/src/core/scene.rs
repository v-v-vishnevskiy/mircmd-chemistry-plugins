use super::Node;

pub struct Scene {
    pub root_node: Node,
}

impl Scene {
    pub fn new() -> Self {
        Self { root_node: Node::new() }
    }
}
