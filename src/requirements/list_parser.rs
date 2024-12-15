use regex::Regex;
use super::{ListItem, ListParser};

/**
 * Parses out the following types of markdown style lists:
 * - [ ]
 * - [x]
 * 1. 
 * 1. [ ]
 * 1. [x]
 * a. 
 * - 
 * *
 * +
 */
const TODO_HEADER: &str = r"(?<todo>- \[(?<todo_mark>.)])";
const ORDERED_HEADER: &str = r"(?<ordered>(?<number>[0-9]+)\.|(?<letter>[a-zA-Z])\.)";
const UNORDERED_HEADER: &str = r"(?<unordered>[-+*])";
const HYBRID_HEADER: &str = r"(?<hybrid>((?<hnumber>[0-9]+)|(?<hletter>[a-zA-Z]))\.\s+\[(?<htodo_mark>.)])";

impl ListParser {
    pub fn new() -> ListParser {
        let body = [HYBRID_HEADER, TODO_HEADER, ORDERED_HEADER, UNORDERED_HEADER].join("|");
        let pattern = format!("^({body})");
        return ListParser(
            Regex::new(&pattern).expect("Could not compile list parser regex.")
        );
    }

    pub fn parse(&self, item: &str) -> Option<(ListItem, String)> {
        let captures = self.0.captures(item)?;

        if let Some(pat) = captures.name("todo") {
            return Some((
                    ListItem::Todo(captures.name("todo_mark").unwrap().as_str().chars().nth(0).unwrap()),
                    item.replace(pat.as_str(), "").trim().to_string()
            ));
        }
        if let Some(pat) = captures.name("ordered") {
            let num: usize;
            if let Some(pat) = captures.name("number") {
                // Should be safe; guaranteed by regex.
                num = pat.as_str().parse::<usize>().unwrap();
            } else {
                let pat = captures.name("letter").unwrap();
                let ascii = *pat.as_str().to_uppercase().as_bytes().get(0).unwrap() as usize;

                // Item should be 1-indexed.
                num = ascii - 64;
            }
            return Some((
                    ListItem::Ordered(num),
                    item.replacen(pat.as_str(), "", 1).trim().to_string()
            ));
        }
        if let Some(pat) = captures.name("unordered") {
            return Some((
                    ListItem::Unordered,
                    item.replace(pat.as_str(), "").trim().to_string()
            ));
        }
        if let Some(pat) = captures.name("hybrid") {
            let ch = captures.name("htodo_mark").unwrap().as_str().chars().nth(0).unwrap();
            let num: usize;
            if let Some(pat) = captures.name("hnumber") {
                // Should be safe; guaranteed by regex.
                num = pat.as_str().parse::<usize>().unwrap();
            } else {
                let pat = captures.name("hletter").unwrap();
                let ascii = *pat.as_str().to_uppercase().as_bytes().get(0).unwrap() as usize;

                // Item should be 1-indexed.
                num = ascii - 64;
            }

            return Some((
                    ListItem::Hybrid(num, ch),
                    item.replace(pat.as_str(), "").trim().to_string()
            ));
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_todo_list() {
        let parser = ListParser::new();

        let res = parser.parse("- [ ] asdf").unwrap();
        assert!(matches!(res.0, ListItem::Todo(' ')));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("- [x] asdf").unwrap();
        assert!(matches!(res.0, ListItem::Todo('x')));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("- [.] asdf").unwrap();
        assert!(matches!(res.0, ListItem::Todo('.')));
        assert_eq!(res.1, "asdf");
    }

    #[test]
    fn parse_ordered_list() {
        let parser = ListParser::new();

        let res = parser.parse("a. asdf").unwrap();
        assert!(matches!(res.0, ListItem::Ordered(1)));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("1. asdf").unwrap();
        assert!(matches!(res.0, ListItem::Ordered(1)));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("16. asdf").unwrap();
        assert!(matches!(res.0, ListItem::Ordered(16)));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("aa. asdf");
        assert!(matches!(res, None));
    }

    #[test]
    fn parse_unordered_list() {
        let parser = ListParser::new();

        let res = parser.parse("- asdf").unwrap();
        assert!(matches!(res.0, ListItem::Unordered));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("+ asdf").unwrap();
        assert!(matches!(res.0, ListItem::Unordered));
        assert_eq!(res.1, "asdf");

        let res = parser.parse("* asdf").unwrap();
        assert!(matches!(res.0, ListItem::Unordered));
        assert_eq!(res.1, "asdf");
    }
    #[test]
    fn parse_hybrid_list() {
        let parser = ListParser::new();

        let res = parser.parse("1. [ ]").unwrap();
        assert!(matches!(res.0, ListItem::Hybrid(1, ' ')));

        let res = parser.parse("2. [>]").unwrap();
        assert!(matches!(res.0, ListItem::Hybrid(2, '>')));

        let res = parser.parse("a. [x]").unwrap();
        assert!(matches!(res.0, ListItem::Hybrid(1, 'x')));
    }
}
