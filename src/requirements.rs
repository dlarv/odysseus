use std::{collections::HashMap, fs::File, hash::{DefaultHasher, Hash, Hasher}, io::Read, path::PathBuf, rc::Rc};

use mythos_core::printerror;
use regex::Regex;

struct RequirementBuilder(Regex, DefaultHasher);

#[derive(Debug, Clone)]
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
            }
            else if tab_level > prev_tab_level {
                // Append item_num to id.
                id.push(item_num);
            }
            else {
                // Pop last item of id and replace.
                id.pop();
                let index = id.len() - 1;
                id[index] = item_num;
            }
            prev_tab_level = tab_level;
            let (val, key) = builder.build(content, &id, category.clone());
            if let Some(collision) = output.insert(key.clone(), val.clone()) {
                printerror!("There was a hash collision while reading the requirements file.");
                printerror!("Original value: {collision:?}");
                printerror!("New value (@line {i}: {val:?}");
                printerror!("Colliding hash: {key:?}");
            }
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

pub fn parse_spreadsheet(path: &PathBuf) -> Option<HashMap<String, Requirement>> {
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
    let mut output: HashMap<String, Requirement> = HashMap::new();

    // Hash,Category,Id,Name,Status
    for (i, line) in contents.split("\n").enumerate() {
        if  line.is_empty() || i == 0 && line == "Hash,Category,Id,Name,Status" {
            continue;
        }

        let values: Vec<&str> = line.split(",").collect();
        let count = values.len();
        if  count != 5 {
            printerror!("Error parsing input spreadsheet on line {i}. There should be 5 items, but found {count}. Line contents: \"{line}\"");
            return None;
        } 
        let (hash, category, id, content, status) = (values[0], values[1], values[2], values[3], values[4]);
        let status = match status.parse::<u8>() {
            Ok(status) => status,
            Err(_) => {
                printerror!("Error parsing input spreadsheet on line {i}. Item #5 should be an integer.");
                return None;
            }
        };
        let req = Requirement {
            category: Rc::new(category.to_string()),
            id: id.to_string(),
            contents: content.to_string(),
            status,
        };
        if let Some(collision) = output.insert(hash.to_string().clone(), req.clone()) {
            printerror!("There was a hash collision while reading the requirements file.");
            printerror!("Original value: {collision:?}");
            printerror!("New value (@line {i}: {hash:?}");
            printerror!("Colliding hash: {req:?}");
        }
    }

    return Some(output);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn try_parse_requirements_file() {
        let reqs = parse_requirements(&PathBuf::from("tests/test.txt")).unwrap();
        println!("{reqs:#?}");
        assert_eq!(reqs.len(), 12);

        // 2. 1.2.2 Item (@hash).
        let req = &reqs["hash"];
        assert_eq!(*req.category, "SYS1".to_string());
        assert_eq!(req.id, "1.2.2".to_string());
        assert_eq!(req.contents, "2. 1.2.2 Item.".to_string());
    }
    #[test]
    fn try_parse_spreadsheet() {
        let reqs = parse_spreadsheet(&PathBuf::from("tests/test.csv")).unwrap();
        println!("{reqs:#?}");

        // H1,CDF,1,This is the third requirement,0
        let req = &reqs["H1"];
        assert_eq!(*req.category, "CDF".to_string());
        assert_eq!(req.id, "1".to_string());
        assert_eq!(req.contents, "This is the third requirement".to_string());
        assert_eq!(req.status, 0);
    }
}
