pub mod requirement_builder;
pub mod requirement;

use std::{collections::HashMap, fs::File, hash::DefaultHasher, io::Read, path::PathBuf, rc::Rc};
use mythos_core::{printerror, printinfo};
use regex::Regex;

struct RequirementBuilder(Regex, DefaultHasher, HashMap<String, String>);

#[derive(Debug, Clone)]
pub struct Requirement {
    pub category: Rc<String>,
    pub hash: String,
    pub id: Vec<usize>,
    pub contents: String,
    pub status: u8
}

pub fn parse_requirements(path: &PathBuf, be_verbose: bool) -> Option<(Vec<Requirement>, HashMap<String, String>)> {
    let contents = match File::open(path) {
        Ok(mut file) => {
            let mut output = String::new();
            if let Err(err) = file.read_to_string(&mut output) {
                printerror!("Could not open requirements file. {err}");
                return None;
            }
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

    let mut output: Vec<Requirement> = Vec::new();
    let mut id: Vec<usize> = vec![1];
    let mut category = Rc::new(String::new());
    let mut prev_tab_level = 0;


    if be_verbose { printinfo!("Reading {path:?}"); }

    for (i, line) in contents.split("\n").enumerate() {
        // Case 1: Skip.
        if line.is_empty() { continue; }
        if be_verbose { printinfo!("Line#{i}: \"{line}\""); }

        let tab_level = line.replace("    ", "\t").matches("\t").count();
        // if be_verbose { printinfo!("Tab level: {0}", line.matches("\t").count()) };

        let mut content = line.trim().to_string();

        let item_num = match parse_list_item(&num_regex, &mut content) {
            Ok(item) => item,
            Err(_) => {
                printerror!("Error parsing item on line {i}.");
                return None;
            },
        };

        // Line has a number prefix.
        if let Some((item_num, fixed_content)) = item_num {
            // Remove item header.
            content = fixed_content;

            // Update id.
            calculate_id(&mut id, prev_tab_level, tab_level, item_num);
            prev_tab_level = tab_level;

            let req = builder.build(content, id.clone(), category.clone());
            output.push(req);
        } 
        // Line has no number prefix.
        else {
            // Case 2: No number => new category.
            category = parse_category(&cat_regex, &content);
            builder.add_new_category(category.clone(), &content);
            id = vec![0];
            prev_tab_level = 0;

            if be_verbose {
                printinfo!("\nAdded new category. Full header: {content}, Abbr: {category}");
            }
        }
    }
    return Some((output, builder.2));
}

fn parse_list_item(regex: &Regex, content: &str) -> Result<Option<(usize, String)>, ()> {
    if let Some(val) = regex.find(&content) {
        let content = content.strip_prefix(val.as_str()).unwrap().trim();
        return match val.as_str().strip_suffix(".").unwrap().parse::<usize>() {
            Ok(index) => {
                Ok(Some((index, content.to_string())))
            },
            Err(_) => {
                return Err(());
            }
        };
    } 
    return Ok(None);
}

fn parse_category(regex: &Regex, content: &String) -> Rc<String> {
    let category = match regex.find(&content) {
        // Unwrap is safe here b/c "()" is part of the regex definition.
        Some(cat) => cat.as_str().strip_suffix(")").unwrap().strip_prefix("(").unwrap().to_string(),
        None => return Rc::new(content.to_string())
    };
    return Rc::new(category);
}

fn calculate_id(id: &mut Vec<usize>, prev_tab_level: usize, curr_tab_level: usize, item_num: usize) {
    // Ignore parsed item_num and simply increment/decrement.
    // This will enable support for unordered lists.

    // Replace last item of id.
    if curr_tab_level == prev_tab_level {
        // TODO: Change this to increment when adding non-ordered list functionality.
        if id.len() == 0 {
            id.push(1);
        } else { 
            let index = id.len() - 1;
            id[index] += 1;
        }
    }
    // Append item_num to id.
    else if curr_tab_level > prev_tab_level {
        id.push(1);
    }
    // Pop last item of id and replace.
    else {
        id.pop();
        if id.len() == 0 {
            id.push(1);
        } else { 
            let index = id.len() - 1;
            id[index] += 1;
        }
    }
}

pub fn parse_spreadsheet(path: &PathBuf, be_verbose: bool) -> Option<HashMap<String, Requirement>> {
    let contents = match File::open(path) {
        Ok(mut file) => {
            let mut output = String::new();
            if let Err(err) = file.read_to_string(&mut output) {
                printerror!("Error reading spreadsheet. \"{err}\".");
                return None;
            };
            output
        },
        Err(err) => {
            printerror!("Could not open requirements file. {err}");
            return None;
        },
    };
    let mut output: HashMap<String, Requirement> = HashMap::new();

    if be_verbose { printinfo!("\nReading {path:?}"); }

    // Hash,Category,Id,Name,Status
    for (i, line) in contents.split("\n").enumerate() {
        if  line.is_empty() || i == 0 && line.starts_with("Hash") { continue; }

        if be_verbose { printinfo!("Line#{i}: \"{line}\""); }

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
                printerror!("Error parsing input spreadsheet on line {i}. Item #5 should be an integer. Line contents: \"{line}\"");
                return None;
            }
        };
        let req = Requirement {
            category: Rc::new(category.to_string()),
            id: id.split(".").map(|x| x.parse::<usize>().unwrap_or(0)).collect(),
            hash: hash.to_string(),
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
        let reqs = parse_requirements(&PathBuf::from("tests/test.txt"), false).unwrap().0;
        // 2. 1.2.2 Item (@hash).
        let req = &reqs[4];
        assert_eq!(*req.category, "SYS1".to_string());
        assert_eq!(req.id_to_string(), "1.2.2".to_string());
        assert_eq!(req.contents, "1.2.2 Item.".to_string());
    }
    #[test]
    fn try_parse_spreadsheet() {
        let reqs = parse_spreadsheet(&PathBuf::from("tests/test.csv"), true).unwrap();

        // H1,CDF,1,This is the third requirement,0
        let req = &reqs["H1"];
        assert_eq!(*req.category, "CDF".to_string());
        assert_eq!(req.id_to_string(), "1".to_string());
        assert_eq!(req.contents, "This is the third requirement".to_string());
        assert_eq!(req.status, 0);
    }
    #[test]
    fn print_to_text() {
        let req = Requirement {
            category: Rc::new("CAT".to_string()),
            hash: "hash".to_string(),
            id: vec![1, 1, 1],
            contents: "contents.".to_string(),
            status: 0,
        };
        assert_eq!(req.to_text_format(), "\t\t1. contents.(@hash)");
    }
    #[test]
    fn print_to_csv() {
        let req = Requirement {
            category: Rc::new("CAT".to_string()),
            hash: "hash".to_string(),
            id: vec![1, 1, 1],
            contents: "contents.".to_string(),
            status: 0,
        };
        // Hash,Category,Id,Name,Status
        assert_eq!(req.to_csv_format(), "hash,CAT,1.1.1,contents.,0\n");
    }
}

