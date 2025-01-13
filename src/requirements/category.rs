use std::sync::LazyLock;

use mythos_core::printwarn;
use regex::Regex;

use super::Category;

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Regex::new(r"\(.*\)$").unwrap();
    let header = r"(?<header>#* )?";
    let body = r"(?<body>.*)";
    let abbr = r"(?<abbr>\(.*\))$?";
    Regex::new(&format!("{header}{body}{abbr}")).unwrap()
});

impl Category {
    pub fn new(line: &str) -> Option<Category> {
        // #* Category (CAT)
        let re = &*REGEX;
        let captures = match re.captures(line) {
            Some(c) => c,
            None => {
                return Some(Category {
                    short_name: None,
                    long_name: line.to_string(),
                    md_header_level: 0,
                });
            }
        };

        let md_header_level = if let Some(val) = captures.name("header") {
            val.as_str().len() as u8 - 1
        } else {
            0
        };

        let long_name = if let Some(val) = captures.name("body") {
            val.as_str().to_string()
        } else {
            return None;
        };

        let short_name = if let Some(val) = captures.name("abbr") {
            Some(val.as_str()
                .strip_prefix("(").unwrap()
                .strip_suffix(")").unwrap().to_string())
        } else {
            None
        };

        return Some(Category { 
            short_name,
            long_name,
            md_header_level
        });
    }

    pub fn to_text_format(&self) -> String {
        let header = if self.md_header_level > 0 {
            vec!["#"; self.md_header_level.into()].join("") + " "
        } else {
            String::new()
        };
        let abbr = if let Some(abbr) = &self.short_name {
            format!("({abbr})")
        } else {
            String::new()
        };
        return format!("{header}{}{abbr}", self.long_name);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_0() {
        let cat = Category::new("Header 1 (H1)").unwrap();
        assert_eq!(cat.short_name.unwrap(), "H1".to_string());
        assert_eq!(cat.long_name, "Header 1 ".to_string());
        assert_eq!(cat.md_header_level, 0);
    }
    #[test]
    fn header_1() {
        let cat = Category::new("# Header 1 (H1)").unwrap();
        assert_eq!(cat.short_name.unwrap(), "H1".to_string());
        assert_eq!(cat.long_name, "Header 1 ".to_string());
        assert_eq!(cat.md_header_level, 1);
    }
    #[test]
    fn header_8() {
        let cat = Category::new("######## Header 1 (H1)").unwrap();
        assert_eq!(cat.short_name.unwrap(), "H1".to_string());
        assert_eq!(cat.long_name, "Header 1 ".to_string());
        assert_eq!(cat.md_header_level, 8);
    }
    #[test]
    fn malformed_header() {
        let cat = Category::new("########Header 1 (H1)").unwrap();
        assert_eq!(cat.short_name.unwrap(), "H1".to_string());
        assert_eq!(cat.long_name, "########Header 1 ".to_string());
        assert_eq!(cat.md_header_level, 0);
    }
    #[test]
    fn to_text_format() {
        let cat = Category::new("# Header 1 (H1)").unwrap();
        assert_eq!(cat.to_text_format(), "# Header 1 (H1)");
    }

}
