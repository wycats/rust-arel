use arel::nodes::SelectCore;
use arel::nodes;

node!(SelectStatement {
    context: nodes::SelectCore,
    pub cores: Vec<nodes::SelectCore>,
    pub orders: Vec<Box<nodes::Node>>,
    pub limit: Option<nodes::Limit>,
    pub offset: Option<nodes::Offset>,
    pub lock: Option<nodes::Literal>
})

impl SelectStatement {
    pub fn build() -> SelectStatement {
        SelectStatement {
            context: SelectCore::build(),
            cores: vec!(),
            orders: vec!(),
            limit: None,
            lock: None,
            offset: None
        }
    }

    pub fn context(&mut self) -> &mut nodes::SelectCore {
        &mut self.context
    }

    pub fn cores(&self) -> Vec<&nodes::SelectCore> {
        self.cores.iter().chain(Some(&self.context).into_iter()).collect()
    }
}
