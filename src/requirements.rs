use std::{collections::HashMap, fs::File, hash::{DefaultHasher, Hash, Hasher}, io::Read, path::PathBuf, rc::Rc};

use mythos_core::printerror;
use regex::Regex;

struct RequirementBuilder(Regex, DefaultHasher);

#[derive(Debug)]
pub struct Requirement {
    pub category: Rc<String>,
    pub id: String,
    pub contents: String,
    pub status: u8
}

impl RequirementBuilder {
    pub fn new() -> RequirementBuilder {
        // (@<hash>)
        return RequirementBuilder(Regex::new(r"\(@\S*\)$").unwrap(), DefaultHasher::new());
    }
    pub fn build(&mut self, contents: String, id: &Vec<usize>, category: Rc<String>) -> (Requirement, String) {
        let id: String = id.into_iter().fold(String::new(), |acc, x| format!("{acc}.{x}")).trim_matches('.').to_string();
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

        return (
            Requirement { 
                category, 
                id, 
                contents: content,
                status: 0 
            }, 
            hash);
    }
}

pub fn parse_requirements(path: &PathBuf) -> Option<HashMap<String, Requirement>> {
    let contents = match File::open(path) {
        Ok(mut file) => {
            let mut output = String::new();
            file.read_to_string(&mut output);
            output
        },
        Err(err) => {
            printerror!("Could not open requirements file. {err}");
            return None;
        },
    };
    let num_regex = Regex::new(r"^[0-9]+\.").unwrap();
    let cat_regex =  Regex::new(r"\(.*\)$").unwrap();
    let mut builder = RequirementBuilder::new();

    let mut output: HashMap<String, Requirement> = HashMap::new();
    let mut id: Vec<usize> = Vec::new();
    let mut category = Rc::new(String::new());
    let mut prev_tab_level = 0;
    /*
     * Category (Cat_ID)
     *  1. Tabx1, id=1
     *      1. Tabx2, id= 1.1
     *      2. Tabx2, id= 1.2
     *  2. Tabx1, id=2
     */

    for (i, line) in contents.split("\n").enumerate() {
        // Case 1: Skip.
        if line.is_empty() { continue; }

        let tab_level = line.matches("\t").count();
        let content = line.trim().to_string();

        let item_num = match parse_item_num(&num_regex, &content) {
            Ok(item) => item,
            Err(_) => {
                printerror!("Error parsing item on line {i}.");
                return None;
            },
        };

        // Line has a number prefix.
        if let Some(item_num) = item_num {
            if tab_level == prev_tab_level {
                // Replace last item of id.
                if id.len() == 0 {
                    id.push(item_num);
                } else {
                    let index = id.len() - 1;
                    id[index] = item_num;
                }

                let (val, key) = builder.build(content, &id, category.clone());
                output.insert(key, val);
            }
            else if tab_level > prev_tab_level {
                // Append item_num to id.
                id.push(item_num);
                let (val, key) = builder.build(content, &id, category.clone());
                output.insert(key, val);
            }
            else {
                // Pop last item of id and replace.
                id.pop();
                let index = id.len() - 1;
                id[index] = item_num;
                let (val, key) = builder.build(content, &id, category.clone());
                output.insert(key, val);
            }
            prev_tab_level = tab_level;
        } 
        // Line has no number prefix.
        else {
            // Case 2: No number => new category.
            category = parse_category(&cat_regex, content);
            id = Vec::new();
            prev_tab_level = 0;
        }
    }
    return Some(output);
}

fn parse_item_num(regex: &Regex, content: &str) -> Result<Option<usize>, ()> {
    if let Some(val) = regex.find(&content) {
        return match val.as_str().strip_suffix(".").unwrap().parse::<usize>() {
            Ok(index) => Ok(Some(index)),
            Err(_) => {
                return Err(());
            }
        };
    } 
    return Ok(None);
}

fn parse_category(regex: &Regex, content: String) -> Rc<String> {
    let category = match regex.find(&content) {
        // Unwrap is safe here b/c "()" is part of the regex definition.
        Some(cat) => cat.as_str().strip_suffix(")").unwrap().strip_prefix("(").unwrap().to_string(),
        None => return Rc::new(content)
    };
    return Rc::new(category);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn try_parse_file() {
        let reqs = parse_requirements(&PathBuf::from("tests/reqs.txt")).unwrap();
        println!("{reqs:#?}");
        assert_eq!(reqs.len(), 12);
        let req = &reqs["hash"];
		// 2. 1.2.2 Item (@hash).
        assert_eq!(*req.category, "SYS1".to_string());
        assert_eq!(req.id, "1.2.2".to_string());
        assert_eq!(req.contents, "2. 1.2.2 Item.".to_string());
    }
}
