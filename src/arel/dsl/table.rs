use arel::nodes::{QualifiedColumn, ToProjections};
use arel::dsl::select::SelectBuilder;

pub struct Table {
    name: String
}

impl Table {
    pub fn new<S: Str>(string: S) -> Table {
        Table { name: string.as_slice().to_str() }
    }

    pub fn get_name<'a>(&'a self) -> &'a str {
        self.name.as_slice()
    }

    pub fn project<'a, P: ToProjections>(&self, projections: P) -> SelectBuilder {
        let mut select = from(self);
        select.project(projections.to_projections());
        select
    }
}

fn from(table: &Table) -> SelectBuilder {
    SelectBuilder::new(table)
}

impl Index<&'static str, QualifiedColumn> for Table {
    fn index(&self, rhs: &&'static str) -> QualifiedColumn {
        QualifiedColumn {
            relation: self.name.clone(),
            name: rhs.to_str()
        }
    }
}
