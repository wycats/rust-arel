use arel::dsl::Table;
use arel::nodes;
use arel::nodes::{TableName, ToNode, ToOrder, ToProjections, Literal, InnerJoin, Relation};
use arel::nodes::{Join, Subselect, Node};

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

    pub fn from_node<N: ToNode>(node: N) -> SelectBuilder {
        let mut builder = SelectBuilder {
            ast: nodes::SelectStatement::build()
        };

        builder.context().set_left(node);
        builder
    }

    pub fn statement(&self) -> &nodes::SelectStatement {
        &self.ast
    }

    pub fn context(&mut self) -> &mut nodes::SelectCore {
        self.ast.context()
    }

    pub fn from<S: Str>(&mut self, table: S) -> &mut SelectBuilder {
        self.context().set_left(TableName { name: table.as_slice().to_string() });
        self
    }

    pub fn alias<S: Str>(self, alias: S) -> Subselect {
        Subselect::build(self.ast).alias(alias)
    }

    pub fn select(self) -> SelectBuilder {
        let mut builder = SelectBuilder {
            ast: nodes::SelectStatement::build()
        };

        builder.context().set_left(self.ast);
        builder
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

    pub fn offset(mut self, offset: uint) -> SelectBuilder {
        self.ast.offset = Some(nodes::Unary::build(Literal::new(offset.to_string())));
        self
    }

    pub fn where<T: ToNode>(mut self, node: T) -> SelectBuilder {
        self.context().add_where(node.to_node());
        self
    }

    pub fn project<P: ToProjections>(mut self, projections: P) -> SelectBuilder {
        self.context().set_projections(projections.to_projections());
        self
    }

    pub fn join<T: Relation>(mut self, relation: T) -> SelectBuilder {
        self.context().add_join(Join::build(nodes::InnerJoin, relation));
        self
    }

    pub fn outer_join<T: Relation>(mut self, relation: T) -> SelectBuilder {
        self.context().add_join(Join::build(nodes::OuterJoin, relation));
        self
    }

    pub fn right_outer_join<T: Relation>(mut self, relation: T) -> SelectBuilder {
        self.context().add_join(Join::build(nodes::RightOuterJoin, relation));
        self
    }

    pub fn on<T: ToNode>(mut self, on: T) -> SelectBuilder {
        self.context().on(on);
        self
    }
}

impl ToOrder for &'static str {
    fn to_order(self) -> Box<nodes::Node> {
        box nodes::UnqualifiedColumn::new(self) as Box<nodes::Node>
    }
}
