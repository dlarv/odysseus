use super::{RequirementBuilder, Requirement};

use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, rc::Rc};
use regex::Regex;

impl RequirementBuilder {
    pub fn new() -> RequirementBuilder {
        // (@<hash>)
        return RequirementBuilder(Regex::new(r"\(@\S*\)$").unwrap(), DefaultHasher::new(), HashMap::new());
    }
    pub fn build(&mut self, contents: String, id: Vec<usize>, category: Rc<String>, status: char) -> Requirement {
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
                status: self.map_char_to_status(status)
            };
    }
    pub fn add_new_category(&mut self, key: Rc<String>, val: &String) {
        self.2.insert(key.to_string(), val.clone());
    }

    fn map_char_to_status(&self, ch: char) -> u8 {
        return match ch {
            ' ' => 0,
            'x' => 1,
            _ => ch as u8
        };
    }
}

