use arel::nodes;
use arel::nodes::{ToNode, Orderable};

pub trait Predications : ToNode {
    fn eql<N: ToNode>(self, other: N) -> nodes::Equality {
        nodes::Binary::build(self, other)
    }

    fn matches<N: ToNode>(self, other: N) -> nodes::Matches {
        nodes::Binary::build(self, other)
    }

    fn does_not_match<N: ToNode>(self, other: N) -> nodes::DoesNotMatch {
        nodes::Binary::build(self, other)
    }

    fn gt<N: ToNode>(self, other: N) -> nodes::GreaterThan {
        nodes::Binary::build(self, other)
    }

    fn lt<N: ToNode>(self, other: N) -> nodes::LessThan {
        nodes::Binary::build(self, other)
    }

    fn gte<N: ToNode>(self, other: N) -> nodes::GreaterThanOrEqual {
        nodes::Binary::build(self, other)
    }

    fn lte<N: ToNode>(self, other: N) -> nodes::LessThanOrEqual {
        nodes::Binary::build(self, other)
    }
}

impl<N: ToNode> Predications for N {}

pub trait OrderPredications : ToNode {
    fn asc(self) -> nodes::Ascending {
        nodes::Unary::build(self)
    }

    fn desc(self) -> nodes::Descending {
        nodes::Unary::build(self)
    }
}

impl<N: Orderable> OrderPredications for N {}
