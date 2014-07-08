use arel::dsl::Table;
use arel::nodes;
use arel::nodes::{TableName, ToNode, Projection};

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
}

