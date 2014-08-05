use arel::nodes::{Node, ToNode};

node!(Literal {
    pub value: String
})

impl Literal {
    pub fn new<S: Str>(string: S) -> Literal {
        Literal { value: string.as_slice().to_string() }
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
    ($name:ty => $kind:ident) => (
        impl ToBind for $name {
            fn to_bind(self) -> Bind {
                Bind { value: $kind(self) }
            }
        }

        impl ToNode for $name {
            fn to_node(self) -> Box<Node> {
                box self.to_bind() as Box<Node>
            }
        }
    )
)

bind!(uint => UintKind)
bind!(int => IntKind)
bind!(f32 => F32Kind)
bind!(f64 => F64Kind)
bind!(bool => BoolKind)
bind!(String => StringKind)

impl<'a> ToBind for &'a str {
    fn to_bind(self) -> Bind {
        Bind { value: StringKind(self.to_string()) }
    }
}

impl<'a> ToNode for &'a str {
    fn to_node(self) -> Box<Node> {
        box self.to_bind() as Box<Node>
    }
}

impl Bind {
    pub fn from<T: ToBind>(val: T) -> Bind {
        val.to_bind()
    }
}
