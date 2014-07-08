use arel::nodes::{Node, ToNode, ToBorrowedNode, Projection};

node!(SelectCore {
    source: Option<JoinSource>,
    projections: Vec<Box<Projection>>
})

impl SelectCore {
    pub fn build() -> SelectCore {
        SelectCore {
            source: Some(JoinSource::build()),
            projections: vec!()
        }
    }

    pub fn source<'a>(&'a self) -> Option<&'a JoinSource> {
        self.source.as_ref()
    }

    pub fn projections<'a>(&'a self) -> &'a [Box<Projection>] {
        self.projections.as_slice()
    }

    pub fn set_projections(&mut self, projections: Vec<Box<Projection>>) {
        self.projections = projections;
    }

    pub fn set_left<N: ToNode>(&mut self, node: N) {
        match self.source {
            Some(ref mut source) => source.left = Some(node.to_node()),
            None => ()
        }
    }

    pub fn set_right<N: ToNode>(&mut self, node: N) {
        match self.source {
            Some(ref mut source) => source.right = Some(node.to_node()),
            None => ()
        }
    }
}

node!(JoinSource {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>
})

impl JoinSource {
    pub fn left<'a>(&'a self) -> Option<&'a Node> {
        self.left.as_ref().map(|node| {
            node.to_borrowed_node()
        })
    }
    pub fn build() -> JoinSource {
        JoinSource { left: None, right: None }
    }
}
