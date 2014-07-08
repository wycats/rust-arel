use arel::nodes;
use arel::nodes::{Node, Literal, Binary};
use arel::collector::CollectSql;
use arel::visitor::Visitor;

pub struct ToSqlVisitor;

impl Visitor for ToSqlVisitor {
    fn Function(&self, function: &nodes::Function, collector: &mut CollectSql) {
        collector.push(function.name());
        if function.name() == "EXISTS" { collector.push(" ") }
        collector.push("(");
        if function.distinct { collector.push("DISTINCT "); }
        self.fold_join(function.expressions.as_slice(), collector, ", ");
        collector.push(")");

        function.alias.as_ref().map(|alias| {
            collector.push(" AS ");
            alias.visit(self, collector);
        });
    }

    fn Bind(&self, bind: &nodes::Bind, collector: &mut CollectSql) {
        collector.add_bind(bind)
    }

    fn Literal(&self, literal: &nodes::Literal, collector: &mut CollectSql) {
        collector.push(literal.value.as_slice())
    }

    fn UnqualifiedColumn(&self, column: &nodes::UnqualifiedColumn, collector: &mut CollectSql) {
        self.column(column.name.as_slice(), collector);
    }

    fn QualifiedColumn(&self, column: &nodes::QualifiedColumn, collector: &mut CollectSql) {
        self.column(column.relation.as_slice(), collector);
        collector.push(".");
        self.column(column.name.as_slice(), collector);
    }

    fn Equality(&self, equality: &nodes::Equality, collector: &mut CollectSql) {
        self.binary(equality, "=", collector);
    }

    fn Limit(&self, limit: &nodes::Limit, collector: &mut CollectSql) {
        collector.push("LIMIT ");
        limit.operand.visit(self, collector);
    }

    fn Assignment(&self, assign: &nodes::Assignment, collector: &mut CollectSql) {
        self.binary(assign, "=", collector);
    }

    fn And(&self, and: &nodes::And, collector: &mut CollectSql) {
        self.binary(and, "AND", collector);
    }

    fn Or(&self, and: &nodes::Or, collector: &mut CollectSql) {
        self.binary(and, "OR", collector);
    }

    fn Matches(&self, matches: &nodes::Matches, collector: &mut CollectSql) {
        self.binary(matches, "LIKE", collector);
    }

    fn DoesNotMatch(&self, matches: &nodes::DoesNotMatch, collector: &mut CollectSql) {
        self.binary(matches, "NOT LIKE", collector);
    }

    fn Ascending(&self, asc: &nodes::Ascending, collector: &mut CollectSql) {
        self.postfix(asc.operand, " ASC", collector);
    }

    fn Descending(&self, asc: &nodes::Descending, collector: &mut CollectSql) {
        self.postfix(asc.operand, " DESC", collector);
    }

    fn SelectStatement(&self, select: &nodes::SelectStatement, collector: &mut CollectSql) {
        for core in select.cores().iter() {
            core.visit(self, collector)
        }
    }

    fn SelectCore(&self, select: &nodes::SelectCore, collector: &mut CollectSql) {
        collector.push("SELECT");

        let projections = select.projections();
        let last = projections.len() - 1;

        if !projections.is_empty() {
            collector.push(" ");
            for (i, projection) in projections.iter().enumerate() {
                projection.visit(self, collector);
                if i != last { collector.push(", ") }
            }
        }

        select.source().map(|source| {
            collector.push(" FROM ");
            source.visit(self, collector);
        });
    }

    fn JoinSource(&self, select: &nodes::JoinSource, collector: &mut CollectSql) {
        select.left().map(|node| node.visit(self, collector));
    }

    fn TableName(&self, table: &nodes::TableName, collector: &mut CollectSql) {
        self.table(table.name.as_slice(), collector)
    }
}

impl ToSqlVisitor {
    fn binary(&self, binary: &Binary, join: &str, collector: &mut CollectSql) {
        binary.left().visit(self, collector);
        collector.push(" ");
        collector.push(join);
        collector.push(" ");
        binary.right().visit(self, collector);
    }

    fn postfix(&self, node: &Node, suffix: &str, collector: &mut CollectSql) {
        node.visit(self, collector);
        collector.push(suffix);
    }

    fn table(&self, string: &str, collector: &mut CollectSql) {
        self.column(string, collector)
    }

    fn column(&self, string: &str, collector: &mut CollectSql) {
        collector.push("\"");
        collector.push(string);
        collector.push("\"");
    }

    fn fold_join(&self, list: &[Box<Node>], collector: &mut CollectSql, join: &str) {
        let last = list.len() - 1;

        for (i, node) in list.iter().enumerate() {
            node.visit(self, collector);
            if i != last {
                collector.push(join);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arel::nodes;
    use arel::nodes::{Node, ToBorrowedNode, Literal, Bind};
    use arel::collector::SqlCollector;

    pub fn star() -> nodes::Literal {
        nodes::Literal::new("*")
    }

    pub fn foo() -> nodes::Literal {
        nodes::Literal::new("foo")
    }

    fn to_sql<N: Node>(node: N) -> String {
        let mut collector = SqlCollector::new();
        node.visit(&ToSqlVisitor, &mut collector);
        collector.value().to_str()
    }

    pub fn expect_sql<N: ToBorrowedNode>(node: N, value: &str) {
        assert_eq!(to_sql(node.to_borrowed_node()).as_slice(), value)
    }

    fn node<N: Node + 'static>(node: N) -> Box<Node> {
        box node as Box<Node>
    }

    #[test]
    fn test_named_function() {
        let node = nodes::Function::named("omg", vec![node(star())]);
        expect_sql(node, "omg(*)")
    }

    #[test]
    fn test_eq_predication() {
        use arel::predications::Predications;
        let node = nodes::Function::named("omg", vec![node(star())]);
        expect_sql(node.eql(Bind::from(2u)), "omg(*) = 2")
    }

    #[test]
    fn test_builtin_functions() {
        use arel::nodes::{Sum, Exists, Max, Min, Avg, Count, Function};

        let func = Function::builtin(Sum, vec!(node(star()))).distinct();
        expect_sql(func, "SUM(DISTINCT *)");

        let func = Function::builtin(Exists, vec!(node(star()))).distinct();
        expect_sql(func, "EXISTS (DISTINCT *)");

        let func = Function::builtin(Max, vec!(node(star()))).distinct();
        expect_sql(func, "MAX(DISTINCT *)");

        let func = Function::builtin(Min, vec!(node(star()))).distinct();
        expect_sql(func, "MIN(DISTINCT *)");

        let func = Function::builtin(Count, vec!(node(star()))).distinct();
        expect_sql(func, "COUNT(DISTINCT *)");

        let func = Function::builtin(Avg, vec!(node(star()))).distinct();
        expect_sql(func, "AVG(DISTINCT *)");
    }

    #[test]
    fn test_named_function_with_list() {
        let func = nodes::Function::named("omg", vec![node(star()), node(star())]);
        expect_sql(func, "omg(*, *)");
    }

    mod equality {
        use super::*;
        use arel::predications::Predications;

        #[test]
        fn test_escape_strings() {
            let equality = star().eql("Aaron Patterson");
            expect_sql(equality, "* = 'Aaron Patterson'");
        }

        #[test]
        fn test_boolean() {
            let equality = star().eql(false);
            expect_sql(equality, "* = 'f'");

            let equality = star().eql(true);
            expect_sql(equality, "* = 't'");
        }

        #[test]
        fn test_number() {
            let equality = foo().eql(0u);
            expect_sql(equality, "foo = 0");

            let equality = foo().eql(0i);
            expect_sql(equality, "foo = 0");
        }
    }

    mod predications {
        use super::*;
        use arel::Predications;
        use arel::dsl;

        #[test]
        fn test_matches() {
            let table = dsl::Table::new("users");
            let matches = table["name"].matches("foo%");

            expect_sql(matches, r#""users"."name" LIKE 'foo%'"#);
        }

        #[test]
        fn test_not_matches() {
            let table = dsl::Table::new("users");
            let matches = table["name"].does_not_match("foo%");

            expect_sql(matches, r#""users"."name" NOT LIKE 'foo%'"#);
        }
    }

    mod order_predications {
        use super::*;
        use arel::OrderPredications;
        use arel::dsl;

        #[test]
        fn test_column_asc() {
            let table = dsl::Table::new("users");
            let asc = table["name"].asc();

            expect_sql(asc, r#""users"."name" ASC"#);
        }

        #[test]
        fn test_column_desc() {
            let table = dsl::Table::new("users");
            let asc = table["name"].desc();

            expect_sql(asc, r#""users"."name" DESC"#);
        }
    }

    mod conjunctions {
        use super::*;
        use arel::Conjunctions;
        use arel::dsl;
        use arel::nodes::{UnqualifiedColumn, Assignment};

        #[test]
        fn test_and() {
            let and = foo().and(2u);
            expect_sql(and, "foo AND 2");
        }

        #[test]
        fn test_or() {
            let and = foo().or(2u);
            expect_sql(and, "foo OR 2");
        }

        #[test]
        fn test_assignment() {
            let left = UnqualifiedColumn::new("foo");
            let right = UnqualifiedColumn::new("bar");

            let assign = Assignment { left: box left, right: box right };

            expect_sql(assign, "\"foo\" = \"bar\"");
        }

        #[test]
        fn test_qualified_column_assignment() {
            let table = dsl::Table::new("users");
            let assign = Assignment {
                left: box table["id"],
                right: box table["name"]
            };

            expect_sql(assign, r#""users"."id" = "users"."name""#);
        }

    }

    mod limit {
        use super::*;
        use arel::nodes::{Limit, Bind, ToNode};

        #[test]
        fn quotes_strings() {
            let limit = Limit { operand: Bind::from("omg").to_node() };
            expect_sql(limit, "LIMIT 'omg'");
        }
    }

    mod select {
        use super::*;
        use arel::dsl;

        #[test]
        fn simple_select() {
            let table = dsl::Table::new("users");
            let select = table.project([star()]);

            expect_sql(select.statement(), "SELECT * FROM \"users\"");
        }

        #[test]
        fn select_exists() {
            let table = dsl::Table::new("users");
            let select = table.project([star()]).exists();

            expect_sql(select, "EXISTS (SELECT * FROM \"users\")");
        }

        #[test]
        fn select_exists_alias() {
            let table = dsl::Table::new("users");
            let select = table.project([star()]).exists().as_("foo");

            expect_sql(select, "EXISTS (SELECT * FROM \"users\") AS \"foo\"");
        }
    }
}
