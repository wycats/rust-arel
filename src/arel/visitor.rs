use arel::nodes;
use arel::collector::CollectSql;

macro_rules! item(
    ($item:item) => ($item)
)

macro_rules! visitor(
    ($($name:ident),+) => (
        item!(pub trait Visitor {
            $(
                #[allow(non_snake_case_functions)]
                #[allow(unused_variable)]
                fn $name(&self, node: &nodes::$name, collector: &mut CollectSql) {
                    fail!("Not yet implemented {}", stringify!($name))
                }
            )+
        })
    )
)

visitor!(True, False, Function, In, Equality, Delete, Except,
         Intersect, UnionAll, Or, NotIn, NotEqual, Matches,
         LessThanOrEqual, Union, LessThan, Join, GreaterThanOrEqual,
         GreaterThan, DoesNotMatch, Between, Assignment, As, Extract,
         Descending, Ascending, DistinctOn, Lock, Top, On, Offset,
         Not, Limit, Having, Group, Bin, BindParam, Literal, Bind,
         And, Null, UnqualifiedColumn, QualifiedColumn,
         Multiplication, Division, Addition, Subtraction,
         SelectStatement, SelectCore, JoinSource, TableName, TableAlias)
