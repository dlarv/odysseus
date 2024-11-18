use super::Requirement;

impl Requirement {
    pub fn to_text_format(&self) -> String {
        let tabs = "\t".repeat(self.id.len() - 1);
        let line_num = self.id.last().unwrap_or(&0);
        return format!("{tabs}{line_num}. {0}(@{1})", self.contents, self.hash);
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
}


