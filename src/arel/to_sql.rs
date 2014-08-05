use arel::nodes;
use arel::nodes::{Node, ToBorrowedNode, Literal, Binary};
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

    fn Or(&self, or: &nodes::Or, collector: &mut CollectSql) {
        self.binary(or, "OR", collector);
    }

    fn GreaterThan(&self, gt: &nodes::GreaterThan, collector: &mut CollectSql) {
        self.binary(gt, ">", collector);
    }

    fn GreaterThanOrEqual(&self, gte: &nodes::GreaterThanOrEqual, collector: &mut CollectSql) {
        self.binary(gte, ">=", collector);
    }

    fn LessThan(&self, lt: &nodes::LessThan, collector: &mut CollectSql) {
        self.binary(lt, "<", collector);
    }

    fn LessThanOrEqual(&self, lte: &nodes::LessThanOrEqual, collector: &mut CollectSql) {
        self.binary(lte, "<=", collector);
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

    fn Offset(&self, offset: &nodes::Offset, collector: &mut CollectSql) {
        self.prefix(offset.operand, "OFFSET ", collector);
    }

    fn Subselect(&self, subselect: &nodes::Subselect, collector: &mut CollectSql) {
        collector.push("(");
        subselect.select.visit(self, collector);
        collector.push(")");

        self.maybe_visit(&subselect.alias, collector);
    }

    fn SelectStatement(&self, select: &nodes::SelectStatement, collector: &mut CollectSql) {
        for core in select.cores().iter() {
            core.visit(self, collector)
        }

        if !select.orders.is_empty() {
            collector.push(" ORDER BY ");

            self.fold_join(select.orders.as_slice(), collector, ", ");
        }

        self.maybe_visit(&select.lock, collector);
        self.maybe_visit(&select.offset, collector);
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

        if !select.wheres().is_empty() {
            collector.push(" WHERE ");
            self.fold_join(select.wheres().as_slice(), collector, ", ");
        }
    }

    fn JoinSource(&self, source: &nodes::JoinSource, collector: &mut CollectSql) {
        source.left().map(|node| node.visit(self, collector));

        if !source.right().is_empty() {
            collector.push(" ");
            self.fold_join(source.right(), collector, " ");
        }
    }

    fn Join(&self, source: &nodes::Join, collector: &mut CollectSql) {
        let name = match source.kind {
            nodes::InnerJoin => "INNER JOIN ",
            nodes::OuterJoin => "LEFT OUTER JOIN ",
            nodes::RightOuterJoin => "RIGHT OUTER JOIN ",
            nodes::FullOuterJoin => "FULL OUTER JOIN "
        };

        collector.push(name);
        println!("HI");
        source.relation.visit(self, collector);
        self.maybe_visit(&source.on, collector);
    }

    fn TableName(&self, table: &nodes::TableName, collector: &mut CollectSql) {
        self.table(table.name.as_slice(), collector);
    }

    fn TableAlias(&self, alias: &nodes::TableAlias, collector: &mut CollectSql) {
        self.table(alias.get_table_name(), collector);
        collector.push(" ");
        self.table(alias.name.as_slice(), collector);
    }

    fn On(&self, alias: &nodes::On, collector: &mut CollectSql) {
        collector.push("ON ");
        alias.operand.visit(self, collector);
    }
}

impl ToSqlVisitor {
    fn maybe_visit<T: Node>(&self, node: &Option<T>, collector: &mut CollectSql) {
        node.as_ref().map(|node| {
            collector.push(" ");
            node.visit(self, collector);
        });
    }

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

    fn prefix(&self, node: &Node, prefix: &str, collector: &mut CollectSql) {
        collector.push(prefix);
        node.visit(self, collector);
    }

    fn table(&self, string: &str, collector: &mut CollectSql) {
        self.column(string, collector)
    }

    fn column(&self, string: &str, collector: &mut CollectSql) {
        collector.push("\"");
        collector.push(string);
        collector.push("\"");
    }

    fn fold_join<T: ToBorrowedNode>(&self, list: &[T], collector: &mut CollectSql, join: &str) {
        let last = list.len() - 1;

        for (i, node) in list.iter().enumerate() {
            node.to_borrowed_node().visit(self, collector);
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
        collector.value().to_string()
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
        use arel::nodes::ColumnAt;

        #[test]
        fn test_matches() {
            let table = dsl::Table::new("users");
            let matches = table.at("name").matches("foo%");

            expect_sql(matches, r#""users"."name" LIKE 'foo%'"#);
        }

        #[test]
        fn test_not_matches() {
            let table = dsl::Table::new("users");
            let matches = table.at("name").does_not_match("foo%");

            expect_sql(matches, r#""users"."name" NOT LIKE 'foo%'"#);
        }
    }

    mod order_predications {
        use super::*;
        use arel::OrderPredications;
        use arel::dsl;
        use arel::nodes::ColumnAt;

        #[test]
        fn test_column_asc() {
            let table = dsl::Table::new("users");
            let asc = table.at("name").asc();

            expect_sql(asc, r#""users"."name" ASC"#);
        }

        #[test]
        fn test_column_desc() {
            let table = dsl::Table::new("users");
            let asc = table.at("name").desc();

            expect_sql(asc, r#""users"."name" DESC"#);
        }
    }

    mod conjunctions {
        use super::*;
        use arel::Conjunctions;
        use arel::dsl;
        use arel::nodes::{UnqualifiedColumn, Assignment, ColumnAt};

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
                left: box table.at("id"),
                right: box table.at("name")
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

        #[test]
        fn supports_numbers() {
            let limit = Limit { operand: 10u.to_node() };
            expect_sql(limit, "LIMIT 10")
        }
    }

    mod select {
        use super::*;
        use arel::dsl;
        use arel::dsl::{Table, Select};
        use arel::nodes::{UnqualifiedColumn, ColumnAt};

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

        #[test]
        fn select_lock_update() {
            let table = dsl::Table::new("users");
            let select = table.project([star()]).lock();

            expect_sql(select.statement(), "SELECT * FROM \"users\" FOR UPDATE");
        }

        #[test]
        fn order_by() {
            let select = dsl::Table::new("users").project([star()]).order("foo");
            expect_sql(select.statement(), "SELECT * FROM \"users\" ORDER BY \"foo\"");
        }

        #[test]
        fn order_by_asc_desc() {
            use arel::OrderPredications;
            let table = dsl::Table::new("users");

            let select = table.project([star()]).order(UnqualifiedColumn::new("foo").asc());
            expect_sql(select.statement(), r#"SELECT * FROM "users" ORDER BY "foo" ASC"#);

            let select = table.project([star()]).order(table.at("foo").asc());
            expect_sql(select.statement(), r#"SELECT * FROM "users" ORDER BY "users"."foo" ASC"#);

            let select = table.project([star()]).order(UnqualifiedColumn::new("foo").desc());
            expect_sql(select.statement(), r#"SELECT * FROM "users" ORDER BY "foo" DESC"#);

            let select = table.project([star()]).order(table.at("foo").desc());
            expect_sql(select.statement(), r#"SELECT * FROM "users" ORDER BY "users"."foo" DESC"#);
        }

        #[test]
        fn order_by_multiple() {
            let table = dsl::Table::new("users");

            let select = table.project([star()]).order("foo").order("bar");
            expect_sql(select.statement(), r#"SELECT * FROM "users" ORDER BY "foo", "bar""#);
        }

        #[test]
        fn offset() {
            let table = dsl::Table::new("users");
            let select = table.project([star()]).offset(10);
            expect_sql(select.statement(), r#"SELECT * FROM "users" OFFSET 10"#);
        }

        #[test]
        fn simple_where() {
            use arel::Predications;
            let table = dsl::Table::new("users");
            let select = table.project([star()]).where(table.at("age").gt(12u));
            expect_sql(select.statement(), r#"SELECT * FROM "users" WHERE "users"."age" > 12"#);
        }

        #[test]
        fn select_column_where() {
            use arel::Predications;
            let table = dsl::Table::new("users");
            let select = table.project([table.at("id")]).where(table.at("email").eql("stuff"));
            expect_sql(select.statement(), r#"SELECT "users"."id" FROM "users" WHERE "users"."email" = 'stuff'"#);
        }

        #[test]
        fn simple_join() {
            use arel::Predications;
            let left = Table::new("users");
            let right = left.alias();
            let predicate = left.at("id").eql(right.at("id"));

            let select = left.select()
                             .join(right)
                             .on(predicate);

            expect_sql(select.statement(),
                r#"SELECT FROM "users" INNER JOIN "users" "users_2" ON "users"."id" = "users_2"."id""#);
        }

        #[test]
        fn join_on_projected() {
            use arel::Predications;
            let left = Table::new("users");
            let right = left.alias();
            let predicate = left.at("id").eql(right.at("id"));

            let select = left.select()
                             .join(right)
                             .project([star()])
                             .on(predicate);

            expect_sql(select.statement(),
                r#"SELECT * FROM "users" INNER JOIN "users" "users_2" ON "users"."id" = "users_2"."id""#);
        }

        #[test]
        fn outer_join() {
            use arel::Predications;
            let left = Table::new("users");
            let right = left.alias();
            let predicate = left.at("id").eql(right.at("id"));

            let select = left.select()
                             .outer_join(right)
                             .on(predicate);

            expect_sql(select.statement(),
                r#"SELECT FROM "users" LEFT OUTER JOIN "users" "users_2" ON "users"."id" = "users_2"."id""#);
        }

        #[test]
        fn right_outer_join() {
            use arel::Predications;
            let left = Table::new("users");
            let right = left.alias();
            let predicate = left.at("id").eql(right.at("id"));

            let select = left.select()
                             .right_outer_join(right)
                             .on(predicate);

            expect_sql(select.statement(),
                r#"SELECT FROM "users" RIGHT OUTER JOIN "users" "users_2" ON "users"."id" = "users_2"."id""#);
        }

        #[test]
        fn subselect() {
            let inner = Table::new("zomg")
                .select()
                .project([star()])
                .alias("foo");

            let select = Select::from_node(inner)
                .project([UnqualifiedColumn::new("name")]);

            expect_sql(select.statement(),
                r#"SELECT "name" FROM (SELECT * FROM "zomg") "foo""#);
        }
    }
}
