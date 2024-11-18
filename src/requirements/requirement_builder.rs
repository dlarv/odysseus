use super::{ListItem, Requirement, RequirementBuilder};

use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, rc::Rc};
use regex::Regex;

impl RequirementBuilder {
    pub fn new() -> RequirementBuilder {
        // (@<hash>)
        return RequirementBuilder(Regex::new(r"\(@\S*\)$").unwrap(), DefaultHasher::new(), HashMap::new());
    }
    pub fn build(&mut self, contents: String, id: Vec<usize>, category: Rc<String>, list_item: ListItem) -> Requirement {
        let content;
        let hash = match self.0.find(&contents) {
            Some(hash) => {
                // Read hash from end of list item.
                // Remove this value from contents.
                let output = hash.as_str().to_string();
                content = contents.replace(&output, "").trim().to_string();
                // Get index of closing ')', this will either be -1 or -2.
                let end_index = output.len() - 1;
                output[2..end_index].to_string()
            },
            None => {
                contents.hash(&mut self.1);
                let hash = self.1.finish();
                content = contents;
                format!("{hash}")
            }
        };

        return Requirement { 
                category, 
                id, 
                hash,
                contents: content,
                status: RequirementBuilder::map_char_to_status(&list_item),
                list_item,
            };
    }
    pub fn add_new_category(&mut self, key: Rc<String>, val: &String) {
        self.2.insert(key.to_string(), val.clone());
    }

    pub fn map_char_to_status(list_item: &ListItem) -> u8 {
        let ch = match list_item {
            ListItem::Todo(ch) => ch,
            _ => return 0
        };
        return match ch {
            ' ' => 0,
            'x' => 1,
            _ => *ch as u8
        };
    }
    pub fn map_status_to_char(ch: u8) -> char {
        return match ch {
            0 => ' ',
            1 => 'x',
            _ => ch as char
        };
    }
}


