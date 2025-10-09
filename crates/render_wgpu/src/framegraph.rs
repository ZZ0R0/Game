use crate::wgpu;

#[derive(Default)]
pub struct FrameGraph {
    pub(crate) nodes: Vec<Node>,
}

pub(crate) enum Node {
    Clear(wgpu::Color),
    Scene,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(mut self, color: wgpu::Color) -> Self {
        self.nodes.push(Node::Clear(color));
        self
    }
    pub fn scene(mut self) -> Self {
        self.nodes.push(Node::Scene);
        self
    }
}
