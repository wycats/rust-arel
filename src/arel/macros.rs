#[macro_escape]
macro_rules! item(
    ($item:item) => ($item)
)

#[macro_escape]
macro_rules! node(
    ($name:ident, $($rest:ident),+) => (
        node!($name)
        node!($($rest),+)
    );
    ($name:ident) => (
        pub struct $name;
        node_impl!($name)
    );
    ($name:ident $tt:tt) => (
        item!(pub struct $name $tt)
        node_impl!($name)
    )
)

#[macro_escape]
macro_rules! node_is(
    ($name:ident : $trt:ident, $(rest:ident),+) => (
        node_is!($name : $trt)
        node_is!($name : $($rest),+)
    );
    ($name:ident : $trt:ident) => {
        impl ::arel::nodes::$trt for $name {}
    }
)

#[macro_escape]
macro_rules! orderable(
    ($($name:ident),+) => (
        $(node_is!($name : Orderable))+
        $(order_by!($name))+
    )
)

#[macro_escape]
macro_rules! relation(
    ($($name:ident),+) => (
        $(node_is!($name : Relation))+
    )
)

#[macro_escape]
macro_rules! order_by(
    ($($name:ident),+) => (
        $(node_is!($name : OrderBy))+

        $(
            impl ::arel::nodes::ToOrder for $name {
                fn to_order(self) -> Box<::arel::nodes::Node> {
                    box self as Box<::arel::nodes::Node>
                }
            }
        )+
    )
)

#[macro_escape]
macro_rules! infix(
    ($($name:ident),+) => (
        $(node_is!($name : InfixOperation))+
    )
)

#[macro_escape]
macro_rules! projection(
    ($($name:ident),+) => (
        $(node_is!($name : Projection))+

        $(
            impl ::arel::nodes::ToProjection for $name {
                fn to_projection(self) -> Box<::arel::nodes::Projection> {
                    box self as Box<::arel::nodes::Projection>
                }
            }
        )+
    )
)

#[macro_escape]
macro_rules! node_impl(
    ($name:ident) => (
        impl ::arel::nodes::Node for $name {
            fn visit(&self, visitor: &::arel::visitor::Visitor, collector: &mut ::arel::collector::CollectSql) {
                visitor.$name(self, collector)
            }
        }

        impl ::arel::nodes::ToNode for Box<$name> {
            fn to_node(self) -> Box<::arel::nodes::Node + 'static> {
                self as Box<::arel::nodes::Node + 'static>
            }
        }

        impl ::arel::nodes::ToNode for $name {
            fn to_node(self) -> Box<::arel::nodes::Node + 'static> {
                box self as Box<::arel::nodes::Node + 'static>
            }
        }

        impl ::arel::nodes::ToBorrowedNode for $name {
            fn to_borrowed_node(&self) -> &::arel::nodes::Node {
                self as &::arel::nodes::Node
            }
        }

        impl<'a> ::arel::nodes::ToBorrowedNode for &'a $name {
            fn to_borrowed_node(&self) -> &::arel::nodes::Node {
                *self as &::arel::nodes::Node
            }
        }
    )
)

#[macro_escape]
macro_rules! unary(
    ($name:ident, $($rest:ident),+) => (
        unary!($name)
        unary!($($rest),+)
    );
    ($name:ident) => (
        unary!($name {
            pub operand: Box<Node>
        })
    );
    ($name:ident $tt:tt) => (
        node!($name $tt)

        impl Unary for $name {
            fn build<N: ::arel::nodes::ToNode>(operand: N) -> $name {
                $name { operand: operand.to_node() }
            }

            fn operand(&self) -> &Node { &*self.operand }
        }
    )
)

macro_rules! binary(
    ($name:ident, $($rest:ident),+) => (
        binary!($name)
        binary!($($rest),+)
    );
    ($name:ident) => (
        node!($name {
            pub left: Box<::arel::nodes::Node>,
            pub right: Box<::arel::nodes::Node>
        })

        impl Binary for $name {
            fn build<N1: ::arel::nodes::ToNode, N2: ::arel::nodes::ToNode>(left: N1, right: N2) -> $name {
                $name {
                    left: left.to_node(),
                    right: right.to_node()
                }
            }

            fn left(&self) -> &::arel::nodes::Node { &*self.left }
            fn right(&self) -> &::arel::nodes::Node { &*self.right }
        }
    )
)
