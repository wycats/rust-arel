use arel::nodes;
use arel::nodes::{Node, ToNode};

pub trait Conjunctions : ToNode {
    fn and<N: ToNode>(self, other: N) -> nodes::And {
        nodes::And {
            left: self.to_node(),
            right: other.to_node()
        }
    }

    fn or<N: ToNode>(self, other: N) -> nodes::Or {
        nodes::Or {
            left: self.to_node(),
            right: other.to_node()
        }
    }
}

impl<N: Node + ToNode> Conjunctions for N {}
