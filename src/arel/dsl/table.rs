use arel::nodes::{QualifiedColumn, TableName, TableAlias, ColumnAt, ToProjections};
use arel::dsl::select::SelectBuilder;

pub struct Table {
    name: String
}

impl Table {
    pub fn new<S: Str>(string: S) -> Table {
        Table { name: string.as_slice().to_string() }
    }

    pub fn get_name<'a>(&'a self) -> &'a str {
        self.name.as_slice()
    }

    pub fn project<'a, P: ToProjections>(&self, projections: P) -> SelectBuilder {
        let select = from(self);
        select.project(projections.to_projections())
    }

    pub fn select(&self) -> SelectBuilder {
        from(self)
    }

    pub fn alias(&self) -> TableAlias {
        self.alias_as(format!("{}_2", self.name).as_slice())
    }

    pub fn alias_as(&self, alias_name: &str) -> TableAlias {
        let table_name = TableName::build(self.name.as_slice());
        TableAlias::build(table_name, alias_name)
    }
}

fn from(table: &Table) -> SelectBuilder {
    SelectBuilder::new(table)
}

impl ColumnAt for Table {
    fn at<S: Str>(&self, col: S) -> QualifiedColumn {
        QualifiedColumn {
            relation: self.name.clone(),
            name: col.as_slice().to_string()
        }
    }
}
