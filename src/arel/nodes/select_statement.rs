use arel::nodes::SelectCore;
use arel::nodes;

node!(SelectStatement {
    context: nodes::SelectCore,
    pub cores: Vec<nodes::SelectCore>,
    pub orders: Option<Box<nodes::Ordering>>,
    pub limit: Option<nodes::Limit>,
    pub offset: Option<nodes::Offset>
})

impl SelectStatement {
    pub fn build() -> SelectStatement {
        SelectStatement {
            context: SelectCore::build(),
            cores: vec!(),
            orders: None,
            limit: None,
            offset: None
        }
    }

    pub fn context<'a>(&'a mut self) -> &'a mut nodes::SelectCore {
        &mut self.context
    }

    pub fn cores<'a>(&'a self) -> Vec<&'a nodes::SelectCore> {
        self.cores.iter().collect::<Vec<& nodes::SelectCore>>().append_one(&self.context)
    }
}
