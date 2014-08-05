use arel::visitor::Visitor;
use arel::collector::CollectSql;

pub use self::sql_literal::{Literal, Bind};
pub use self::select_core::{SelectCore, JoinSource};
pub use self::select_statement::SelectStatement;

impl ToNode for Box<Node> {
    fn to_node(self) -> Box<Node> {
        self
    }
}

impl ToBorrowedNode for Box<Node> {
    fn to_borrowed_node<'a>(&'a self) -> &'a Node {
        let node: &Node = *self;
        node
    }
}

impl<'a> ToBorrowedNode for &'a Node {
    fn to_borrowed_node<'a>(&'a self) -> &'a Node {
        *self
    }
}

pub trait ToNode {
    fn to_node(self) -> Box<Node>;
}

pub trait ToBorrowedNode {
    fn to_borrowed_node<'a>(&'a self) -> &'a Node;
}

pub trait Node {
    fn visit(&self, visitor: &Visitor, collector: &mut CollectSql);

    fn borrow<'a>(&'a self) -> &'a Node {
        self as &Node
    }
}

impl Node for Box<Node> {
    fn visit(&self, visitor: &Visitor, collector: &mut CollectSql) {
        self.visit(visitor, collector)
    }

    fn borrow<'a>(&'a self) -> &'a Node {
        let node: &Node = self;
        node
    }
}

impl<'a> Node for &'a Node {
    fn visit(&self, visitor: &Visitor, collector: &mut CollectSql) {
        self.visit(visitor, collector)
    }

    fn borrow<'a>(&'a self) -> &'a Node {
        *self
    }
}

pub trait Orderable : Node + ToNode {}
pub trait OrderBy : Node + ToNode {}
pub trait InfixOperation : Node + ToNode + Orderable {}
pub trait Projection : Node + ToNode {}
pub trait Relation : Node + ToNode {}

pub trait ToOrder {
    fn to_order(self) -> Box<Node>;
}

pub trait ToProjection {
    fn to_projection(self) -> Box<Projection>;
}

impl ToProjection for Box<Projection> {
    fn to_projection(self) -> Box<Projection> {
        self
    }
}

pub trait ToProjections {
    fn to_projections(self) -> Vec<Box<Projection>>;
}

impl<P: ToProjection> ToProjections for Vec<P> {
    fn to_projections(self) -> Vec<Box<Projection>> {
        self.move_iter().map(|p| p.to_projection()).collect()
    }
}

impl<P: ToProjection> ToProjections for [P, ..1] {
    fn to_projections(self) -> Vec<Box<Projection>> {
        vec!(self[0].to_projection())
    }
}

impl<P: ToProjection> ToProjections for (P, P) {
    fn to_projections(self) -> Vec<Box<Projection>> {
        let (a, b) = self;
        vec!(a.to_projection(), b.to_projection())
    }
}

impl<P: ToProjection> ToProjections for (P, P, P) {
    fn to_projections(self) -> Vec<Box<Projection>> {
        let (a, b, c) = self;
        vec!(a.to_projection(), b.to_projection(), c.to_projection())
    }
}

node!(False, True, Null)

node!(QualifiedColumn {
    pub relation: String,
    pub name: String
})

orderable!(QualifiedColumn)
projection!(QualifiedColumn)

node!(UnqualifiedColumn {
    pub name: String
})

orderable!(UnqualifiedColumn)
projection!(UnqualifiedColumn)

impl UnqualifiedColumn {
    pub fn new<S: Str>(string: S) -> UnqualifiedColumn {
        UnqualifiedColumn { name: string.as_slice().to_string() }
    }
}

node!(TableName {
    pub name: String
})

relation!(TableName)

impl TableName {
    pub fn build(name: &str) -> TableName {
        TableName { name: name.to_string() }
    }
}

node!(TableAlias {
    pub name: String,
    pub relation: TableName
})

relation!(TableAlias)

impl TableAlias {
    pub fn build(relation: TableName, alias_name: &str) -> TableAlias {
        TableAlias { name: alias_name.to_string(), relation: relation }
    }

    pub fn get_table_name<'a>(&'a self) -> &'a str {
        self.relation.name.as_slice()
    }
}

pub trait ColumnAt {
    fn at<S: Str>(&self, col: S) -> QualifiedColumn;
}

impl ColumnAt for TableAlias {
    fn at<S: Str>(&self, col: S) -> QualifiedColumn {
        QualifiedColumn {
            relation: self.name.clone(),
            name: col.as_slice().to_string()
        }
    }
}
pub mod sql_literal;
pub mod select_statement;
pub mod select_core;

pub enum Direction {
    Asc,
    Desc
}

pub trait Ordering {
    fn reverse(self) -> Box<Ordering>;
    fn direction(&self) -> Direction;

    fn is_ascending(&self) -> bool {
        match self.direction() {
            Asc => true,
            Desc => false
        }
    }

    fn is_descending(&self) -> bool {
        match self.direction() {
            Asc => false,
            Desc => true
        }
    }
}

order_by!(Ascending, Descending)

pub trait Unary {
    fn build<N: ToNode>(operand: N) -> Self;
    fn operand<'a>(&'a self) -> &'a Node;
}

unary!(Bin, Group, Having, Limit, Not, Offset, On, Top, Lock, DistinctOn,
       Ascending, Descending)

node!(Extract {
    pub operand: Box<Node>,
    pub field: String,
    pub alias: Literal
})

impl Ordering for Descending {
    fn reverse(self) -> Box<Ordering> {
        box Ascending { operand: self.operand } as Box<Ordering>
    }

    fn direction(&self) -> Direction { Desc }
}

impl Ordering for Ascending {
    fn reverse(self) -> Box<Ordering> {
        box Descending { operand: self.operand } as Box<Ordering>
    }

    fn direction(&self) -> Direction { Asc }
}

pub trait Binary {
    fn build<N1: ToNode, N2: ToNode>(left: N1, right: N2) -> Self;
    fn left<'a>(&'a self) -> &'a Node;
    fn right<'a>(&'a self) -> &'a Node;
}

binary!(As, Assignment, Between, DoesNotMatch, GreaterThan, GreaterThanOrEqual,
        LessThan, LessThanOrEqual, Matches, NotEqual, NotIn, Or, Union,
        UnionAll, Intersect, Except, Delete, Equality, In, And,
        Multiplication, Division, Addition, Subtraction)

orderable!(Multiplication, Division, Addition, Subtraction)
infix!(Multiplication, Division, Addition, Subtraction)

node!(BindParam {
    pub bind: String
})

pub enum JoinKind {
    InnerJoin,
    OuterJoin,
    FullOuterJoin
}

node!(Join {
    pub kind: JoinKind,
    pub relation: Box<Node>,
    pub on: Option<On>
})

impl Join {
    pub fn build<T: ToNode>(kind: JoinKind, relation: T) -> Join {
        Join { kind: kind, relation: relation.to_node(), on: None }
    }
}

pub enum FunctionKind {
    Sum,
    Exists,
    Max,
    Min,
    Avg,
    Count,
    Named(String)
}

node!(Function {
    pub kind: FunctionKind,
    pub expressions: Vec<Box<Node>>,
    pub alias: Option<UnqualifiedColumn>,
    pub distinct: bool
})

impl Function {
    pub fn named(name: &str, exprs: Vec<Box<Node>>) -> Function {
        Function {
            kind: Named(name.to_string()),
            expressions: exprs,
            alias: None,
            distinct: false
        }
    }

    pub fn builtin(kind: FunctionKind, exprs: Vec<Box<Node>>) -> Function {
        Function {
            kind: kind,
            expressions: exprs,
            alias: None,
            distinct: false
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        match self.kind {
            Sum => "SUM",
            Exists => "EXISTS",
            Max => "MAX",
            Min => "MIN",
            Avg => "AVG",
            Count => "COUNT",
            Named(ref string) => string.as_slice()
        }
    }

    pub fn as_(mut self, alias: &str) -> Function {
        self.alias = Some(UnqualifiedColumn::new(alias));
        self
    }

    pub fn distinct(mut self) -> Function {
        self.distinct = true;
        self
    }
}
