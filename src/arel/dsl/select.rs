use arel::dsl::Table;
use arel::nodes;
use arel::nodes::{TableName, ToNode, ToOrder, Projection};

pub struct SelectBuilder {
    ast: nodes::SelectStatement
}

impl SelectBuilder {
    pub fn new(table: &Table) -> SelectBuilder {
        let mut builder = SelectBuilder {
            ast: nodes::SelectStatement::build()
        };

        builder.from(table.get_name());
        builder
    }

    pub fn statement<'a>(&'a self) -> &'a nodes::SelectStatement {
        &self.ast
    }

    pub fn context<'a>(&'a mut self) -> &'a mut nodes::SelectCore {
        self.ast.context()
    }

    pub fn project<'a>(&'a mut self, projections: Vec<Box<Projection>>) -> &'a mut SelectBuilder {
        self.context().set_projections(projections);
        self
    }

    pub fn from<'a, S: Str>(&'a mut self, table: S) -> &'a mut SelectBuilder {
        self.context().set_left(TableName { name: table.as_slice().to_str() });
        self
    }

    pub fn exists(self) -> nodes::Function {
        nodes::Function::builtin(nodes::Exists, vec!(self.ast.to_node()))
    }

    pub fn lock(mut self) -> SelectBuilder {
        self.ast.lock = Some(nodes::Literal::new("FOR UPDATE"));
        self
    }

    pub fn lock_for(mut self, string: &str) -> SelectBuilder {
        self.ast.lock = Some(nodes::Literal::new(format!("FOR {}", string)));
        self
    }

    pub fn order<T: ToOrder>(mut self, order: T) -> SelectBuilder {
        self.ast.orders.push(order.to_order());
        self
    }
}

impl ToOrder for &'static str {
    fn to_order(self) -> Box<nodes::Node> {
        box nodes::UnqualifiedColumn::new(self) as Box<nodes::Node>
    }
}
