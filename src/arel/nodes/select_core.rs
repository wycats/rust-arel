use arel::nodes::{Node, ToNode, ToBorrowedNode, Projection, Join, Unary};

node!(SelectCore {
    source: Option<JoinSource>,
    wheres: Vec<Box<Node>>,
    projections: Vec<Box<Projection>>
})

impl SelectCore {
    pub fn build() -> SelectCore {
        SelectCore {
            source: Some(JoinSource::build()),
            wheres: vec!(),
            projections: vec!()
        }
    }

    pub fn add_where(&mut self, node: Box<Node>) {
        self.wheres.push(node)
    }

    pub fn source<'a>(&'a self) -> Option<&'a JoinSource> {
        self.source.as_ref()
    }

    pub fn projections<'a>(&'a self) -> &'a [Box<Projection>] {
        self.projections.as_slice()
    }

    pub fn wheres<'a>(&'a self) -> &'a [Box<Node>] {
        self.wheres.as_slice()
    }

    pub fn set_projections(&mut self, projections: Vec<Box<Projection>>) {
        self.projections = projections;
    }

    pub fn add_join(&mut self, node: Join) {
        match self.source {
            Some(ref mut source) => source.right.push(node),
            None => ()
        }
    }

    pub fn on<T: ToNode>(&mut self, node: T) {
        match self.source {
            Some(ref mut source) => source.on(node),
            None => ()
        }
    }

    pub fn set_left<N: ToNode>(&mut self, node: N) {
        match self.source {
            Some(ref mut source) => source.left = Some(node.to_node()),
            None => ()
        }
    }
}

node!(JoinSource {
    left: Option<Box<Node>>,
    right: Vec<Join>
})

impl JoinSource {
    pub fn left<'a>(&'a self) -> Option<&'a Node> {
        self.left.as_ref().map(|node| {
            node.to_borrowed_node()
        })
    }

    pub fn right<'a>(&'a self) -> &'a [Join] {
        self.right.as_slice()
    }

    pub fn on<T: ToNode>(&mut self, on: T) {
        match self.right.mut_last() {
            Some(join) => join.on = Some(Unary::build(on.to_node())),
            None => ()
        }
    }

    pub fn build() -> JoinSource {
        JoinSource { left: None, right: vec!() }
    }
}
