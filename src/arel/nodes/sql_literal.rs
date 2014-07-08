use arel::nodes::{Node, ToNode};

node!(Literal {
    pub value: String
})

impl Literal {
    pub fn new<S: Str>(string: S) -> Literal {
        Literal { value: string.as_slice().to_str() }
    }
}

projection!(Literal)

pub enum BindValue {
    IntKind(int),
    UintKind(uint),
    BoolKind(bool),
    F32Kind(f32),
    F64Kind(f64),
    StringKind(String),
}

node!(Bind {
    pub value: BindValue
})

orderable!(Bind)
projection!(Bind)

pub trait ToBind {
    fn to_bind(self) -> Bind;
}

macro_rules! bind(
    ($name:ty => $kind:ident($expr:expr)) => (
        impl<'a> ToBind for $name {
            fn to_bind(self) -> Bind {
                Bind { value: $kind($expr) }
            }
        }

        impl<'a> ToNode for $name {
            fn to_node(self) -> Box<Node> {
                box self.to_bind() as Box<Node>
            }
        }
    )
)

bind!(uint => UintKind(self))
bind!(int => IntKind(self))
bind!(f32 => F32Kind(self))
bind!(f64 => F64Kind(self))
bind!(bool => BoolKind(self))
bind!(String => StringKind(self))
bind!(&'a str => StringKind(self.to_str()))

impl Bind {
    pub fn from<T: ToBind>(val: T) -> Bind {
        val.to_bind()
    }
}
