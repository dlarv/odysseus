use super::{RequirementBuilder, Requirement};

use std::{collections::HashMap, fs::File, hash::{DefaultHasher, Hash, Hasher}, io::Read, path::PathBuf, rc::Rc};
use mythos_core::{printerror, printinfo};
use regex::Regex;

impl RequirementBuilder {
    pub fn new() -> RequirementBuilder {
        // (@<hash>)
        return RequirementBuilder(Regex::new(r"\(@\S*\)$").unwrap(), DefaultHasher::new(), HashMap::new());
    }
    pub fn build(&mut self, contents: String, id: Vec<usize>, category: Rc<String>) -> Requirement {
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
                status: 0 
            };
    }
    pub fn add_new_category(&mut self, key: Rc<String>, val: &String) {
        self.2.insert(key.to_string(), val.clone());
    }
}
