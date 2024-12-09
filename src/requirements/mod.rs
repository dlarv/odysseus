mod requirement_builder;
mod requirement;
mod list_parser;

use std::{collections::HashMap, fs::File, hash::DefaultHasher, io::Read, path::PathBuf, rc::Rc};
use regex::Regex;
use mythos_core::{printerror, printinfo};

#[derive(Debug, Clone)]
pub enum ListItem { Ordered(usize), Unordered, Todo(char) }

struct ListParser(Regex);

struct RequirementBuilder(Regex, DefaultHasher, HashMap<String, String>);

#[derive(Debug, Clone)]
pub struct Requirement {
    pub category: Rc<String>,
    pub hash: String,
    pub id: Vec<usize>,
    pub contents: String,
    pub list_item: ListItem,
    pub status: u8,
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
    let cat_regex =  Regex::new(r"\(.*\)$").unwrap();
    let mut builder = RequirementBuilder::new();
    let parser = ListParser::new();

    let mut output: Vec<Requirement> = Vec::new();
    let mut id: Vec<usize> = vec![1];
    let mut category = Rc::new(String::new());
    let mut prev_tab_level = 0;


    printinfo!(be_verbose, "Reading {path:?}");

    for (i, line) in contents.split("\n").enumerate() {
        // Case 1: Skip.
        if line.is_empty() { continue; }
        printinfo!(be_verbose, "Line#{i}: \"{line}\"");

        // let tab_level = line.replace("    ", "\t").matches("\t").count();
        let tab_level = count_starting_tabs(&line);
        printinfo!(be_verbose, "Tab level: {0}", line.matches(" ").count());

        let mut content = line.trim().to_string();

        let item_num = parser.parse(&content);

        // Line has a number prefix.
        if let Some((list_item, fixed_content)) = item_num {
            // Remove item header.
            content = fixed_content;

            // Update id.
            calculate_id(&mut id, prev_tab_level, tab_level, &list_item);
            prev_tab_level = tab_level;

            let req = builder.build(content, id.clone(), category.clone(), list_item);
            output.push(req);
        } 
        // Line has no number prefix.
        else {
            // Case 2: No number => new category.
            category = parse_category(&cat_regex, &content);
            builder.add_new_category(category.clone(), &content);
            id = vec![0];
            prev_tab_level = 0;

            printinfo!(be_verbose, "\nAdded new category. Full header: {content}, Abbr: {category}");
        }
    }
    return Some((output, builder.2));
}

fn parse_category(regex: &Regex, content: &String) -> Rc<String> {
    let category = match regex.find(&content) {
        // Unwrap is safe here b/c "()" is part of the regex definition.
        Some(cat) => cat.as_str().strip_suffix(")").unwrap().strip_prefix("(").unwrap().to_string(),
        None => return Rc::new(content.to_string())
    };
    return Rc::new(category);
}

fn calculate_id(id: &mut Vec<usize>, prev_tab_level: usize, curr_tab_level: usize, item_num: &ListItem) {
    // Ignore parsed item_num and simply increment/decrement.
    // This will enable support for unordered lists.

    // Replace last item of id.
    if curr_tab_level == prev_tab_level {
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

/// Count only the beginning whitespace. Tabs and spaces are treated as equal.
fn count_starting_tabs(line: &str) -> usize {
    let mut counter = 0;
    for ch in line.chars() {
        if !ch.is_whitespace() {
            break;
        }
        counter += 1;
    }
    return counter;
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

    printinfo!(be_verbose, "\nReading {path:?}");

    // Detect whether this is a csv file or md.
    let mut use_md_format: bool = false;

    // Hash,Category,Id,Name,Status
    for (i, line) in contents.split("\n").enumerate() {
        if line.is_empty() { continue; }
        if i == 0 {
            if Requirement::check_md_header(&line) {
                printinfo!(be_verbose, "Md header detected: \"{line}\".");
                use_md_format = true;
            } else {
                printinfo!(be_verbose, "Csv header detected: \"{line}\".");
            }
            continue;
        }
        // Second line of md will be "|---|---|---..."
        if i == 1 && use_md_format {
            continue;
        }

        printinfo!(be_verbose, "Line#{i}: \"{line}\"");

        let values: Vec<&str> = if use_md_format {
            parse_md_line(&line, i)?
        } else {
            line.split(",").collect()
        };

        let count = values.len();
        if  count != 5 {
            printerror!("Error parsing input spreadsheet on line {i}. There should be 5 items, but found {count}. Line contents: \"{line}\"");
            return None;
        } 
        let (hash, category, id, content, status) = (values[0], values[1], values[2], values[3], values[4]);

        let status = match parse_csv_status(status) {
            Ok(val) => val,
            Err(_) => {
                printerror!("Error on line {i}. Couldn't parse status. Status = \"{status}\".");
                return None;
            }
        };
        let id: Vec<usize> = id.split(".").map(|x| x.parse::<usize>().unwrap_or(0)).collect(); 

        let req = Requirement {
            category: Rc::new(category.to_string()),
            list_item: ListItem::Ordered(*id.last().unwrap_or(&0)),
            id, 
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
fn parse_csv_status(status: &str) -> Result<u8, ()> {
    match status.parse::<u8>() {
        Ok(status) => return Ok(status),
        Err(_) => (),        
    };

    if status.len() > 1 {
        return Err(());
    }
    return Ok(status.chars().nth(0).unwrap_or(0 as char) as u8);
}

fn parse_md_line<'a>(line: &'a str, i: usize) -> Option<Vec<&'a str>> {
    let line = match line.strip_prefix("|") {
        Some(line) => line,
        None => {
            printerror!("Error parsing input spreadsheet on line {i}. Markdown style tables must begin and end with '|'.");
            return None;
        }
    };
    let line = match line.strip_suffix("|") {
        Some(line) => line,
        None => {
            printerror!("Error parsing input spreadsheet on line {i}. Markdown style tables must begin and end with '|'.");
            return None;
        }
    };

    return Some(line.split("|").collect());
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn try_parse_requirements_file() {
        let reqs = parse_requirements(&PathBuf::from("tests/test.txt"), true).unwrap().0;
        // 2. 1.2.2 Item (@hash).
        let req = &reqs[4];
        assert_eq!(*req.hash, "hash".to_string());
        assert_eq!(*req.category, "SYS1".to_string());
        assert_eq!(req.id_to_string(), "1.2.2".to_string());
        assert_eq!(req.contents, "1.2.2 Item.".to_string());
    }
    #[test]
    fn try_parse_spreadsheet() {
        let reqs = parse_spreadsheet(&PathBuf::from("tests/test.csv"), true).unwrap();

        // H1,CDF,1,This is the third requirement,0
        let req = &reqs["H1"];
        assert_eq!(*req.category, "ABC".to_string());
        assert_eq!(req.id_to_string(), "1".to_string());
        assert_eq!(req.contents, "This is the first req.".to_string());
        assert_eq!(req.status, 0);
    }
    #[test]
    fn try_parse_md_spreadsheet() {
        let reqs = parse_spreadsheet(&PathBuf::from("tests/test.md"), true).unwrap();

        // H1,CDF,1,This is the third requirement,0
        let req = &reqs["H1"];
        assert_eq!(*req.category, "ABC".to_string());
        assert_eq!(req.id_to_string(), "1".to_string());
        assert_eq!(req.contents, "This is the first req.".to_string());
        assert_eq!(req.status, 0);
    }
    #[test]
    fn test_todo_items() {
        let reqs = parse_requirements(&PathBuf::from("tests/test_todo.txt"), true).unwrap().0;
        let csv = parse_spreadsheet(&PathBuf::from("tests/test_todo.csv"), true).unwrap();
        let r1 = &reqs[2];
        let r2 = &csv["h3"];

        // Going from csv to txt loses info. Parser cannot know what kind of list was used, so
        // defaults to basic ordered.
        // assert_eq!(r1.to_text_format(), r2.to_text_format());
        assert_eq!(r1.to_csv_format(), r2.to_csv_format());
    }
    #[test]
    fn print_to_text() {
        let req = Requirement {
            category: Rc::new("CAT".to_string()),
            hash: "hash".to_string(),
            id: vec![1, 1, 1],
            contents: "contents.".to_string(),
            list_item: ListItem::Ordered(1),
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
            list_item: ListItem::Ordered(1),
            status: 0,
        };
        // Hash,Category,Id,Name,Status
        assert_eq!(req.to_csv_format(), "hash,CAT,1.1.1,contents.,0\n");
    }
    #[test]
    fn print_to_md() {
        let req = Requirement {
            category: Rc::new("CAT".to_string()),
            hash: "hash".to_string(),
            id: vec![1, 1, 1],
            contents: "contents.".to_string(),
            list_item: ListItem::Ordered(1),
            status: 0,
        };
        // Hash,Category,Id,Name,Status
        assert_eq!(req.to_md_format(), "|hash|CAT|1.1.1|contents.|0|\n");
    }
}

