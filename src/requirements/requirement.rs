use mythos_core::printinfo;

use super::{ListItem, Requirement, RequirementBuilder};

impl Requirement {
    pub fn to_text_format(&self) -> String {
        let tabs = "\t".repeat(self.id.len() - 1);
        let line_num = match self.list_item {
            super::ListItem::Ordered(num) => format!("{num}."),
            super::ListItem::Unordered => format!("-"),
            super::ListItem::Todo(ch) => format!("- [{ch}]"),
        };
        return format!("{tabs}{line_num} {0}(@{1})", self.contents, self.hash);
    }
    pub fn to_csv_format(&self) -> String {
        // Hash,Category,Id,Name,Status
        return format!("{hash},{cat},{id},{contents},{status}\n", 
            hash=self.hash, 
            cat=self.category,
            id=self.id_to_string(),
            contents=self.contents,
            status=self.status);
    }
    pub const fn get_csv_header() -> &'static str {
        return "Hash,Category,Id,Contents,Status\n";
    }

    pub fn id_to_string(&self) -> String {
        return self.id.iter().fold(String::new(), |acc, x| format!("{acc}.{x}")).trim_matches('.').to_string();
    }

    pub fn copy_status(&mut self, other: &Requirement, be_verbose: bool) {
        if be_verbose && self.status != other.status { 
            printinfo!("Overwriting status with value from csv file: {} -> {}.", self.status, other.status); 
        }

        self.status = other.status;

        if matches!(self.list_item, ListItem::Todo(_)) {
            self.list_item = ListItem::Todo(RequirementBuilder::map_status_to_char(self.status));
        }

    }
}


