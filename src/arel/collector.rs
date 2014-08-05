use arel::nodes::sql_literal::Bind;

pub trait Collect {
    fn push<'a>(&'a mut self, string: &str);
}

pub trait CollectSql : Collect {
    fn add_bind<'a>(&'a mut self, bind: &Bind);
}

pub struct SqlCollector {
    string: String
}

impl SqlCollector {
    pub fn new() -> SqlCollector {
        SqlCollector { string: String::with_capacity(1024) }
    }

    pub fn value<'a>(&'a self) -> &'a str {
        self.string.as_slice()
    }
}

impl Collect for SqlCollector {
    fn push<'a>(&'a mut self, string: &str) {
        self.string.push_str(string);
    }
}

impl CollectSql for SqlCollector {
    fn add_bind<'a>(&'a mut self, bind: &Bind) {
        use arel::nodes::sql_literal;

        let bind = match bind.value {
            sql_literal::UintKind(u) => u.to_string(),
            sql_literal::IntKind(i) => i.to_string(),
            sql_literal::F32Kind(f) => f.to_string(),
            sql_literal::F64Kind(f) => f.to_string(),
            sql_literal::BoolKind(true) => "'t'".to_string(),
            sql_literal::BoolKind(false) => "'f'".to_string(),
            sql_literal::StringKind(ref s) => format!("'{}'", s),
        };

        self.push(bind.as_slice());
    }
}

