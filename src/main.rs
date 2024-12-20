mod requirements;

use std::ffi::OsString;
use std::io::Write;
use std::{collections::HashMap, io::BufWriter};
use std::fs::File;
use std::path::PathBuf;

use mythos_core::logger::set_id;
use mythos_core::printinfo;
use mythos_core::{cli::clean_cli_args, printerror};
use requirements::{parse_requirements, parse_spreadsheet, Requirement};


fn main() -> Result<(), ()>{
    let _ = set_id("ODYSSEUS");
    let mut input_path: Option<PathBuf> = None;
    let input_data: Vec<Requirement>;

    let mut output_path: Option<PathBuf> = None;
    let mut output_data: HashMap<String, Requirement> = HashMap::new();

    let mut overwrite_original_file = true;
    let mut do_dry_run = false;
    let mut be_verbose = false;
    let mut use_markdown_output = false;

    let mut args = clean_cli_args().into_iter().peekable();

    if args.peek().is_none() {
        print_help();
        return Ok(());
    }

    while let Some(arg) = args.next() {
        // Interpret first non-opt arg as the input file.
        if !arg.starts_with("-") {
            let path = PathBuf::from(arg);
            if !path.is_file() {
                printerror!("Input file {path:?} does not exist.");
                return Err(());
            }
            input_path = Some(path);
            break;
        }
        match arg.as_str() {
            "-o" | "--output" => {
                let arg = args.next().unwrap_or("".to_string());
                if arg.is_empty() || arg.starts_with("-") {
                    printerror!("-o/--open arg must be accompanied with a file path.");
                    return Err(());
                }
                output_path = Some(PathBuf::from(arg));
            },
            "-m" | "--markdown" => use_markdown_output = true,
            "-w" | "--no-overwrite" => overwrite_original_file = false,
            "-n" | "--dry-run" => do_dry_run = true,
            "-v" | "--verbose" => be_verbose = true,
            "-h" | "--help" | _ => {
                print_help();
                return Ok(());
            },

        }
    }

    // Ensure user has provided a input file.
    if input_path.is_none() {
        printerror!("User must provide an input path.");
        return Err(());
    }
    let input_path = input_path.unwrap();
    let categories: HashMap<String, String>;
    (input_data, categories) = match parse_requirements(&input_path, be_verbose) {
        Some(data) => data,
        None => return Err(())
    };

    // If there is one more arg, treat it as the output_data.
    if args.peek().is_some() {
        let arg = args.next().unwrap();
        let path = PathBuf::from(arg);

        // If user did not override output path, use provided spreadsheet path.
        if output_path.is_none() {
            output_path = Some(path);
        }
    }


    // If user did not provide a -o arg or a spreadsheet file, output to ./<input_file_name>.csv.
    let output_path = if output_path.is_none() {
        printinfo!("No previous csv file provided.");
        PathBuf::from(input_path.clone().parent().unwrap_or(PathBuf::from(".").as_path())
            .file_stem()
            .unwrap_or(&OsString::from("requirements")))
            .with_extension(
                if use_markdown_output {
                    "csv.md"
                } else {
                    "csv"
                })
    } else {
        let o = output_path.unwrap();
        output_data = match parse_spreadsheet(&o, be_verbose) {
            Some(data) => data,
            None => return Err(())
        };
        printinfo!("Previous csv file provided. Reading from {o:?}.");
        o

    };
    println!("{output_data:?}");

    // Writer to spreadsheet file.
    let mut spreadsheet_writer = BufWriter::new(match File::create(&output_path) {
        Ok(writer) => writer,
        Err(err) => {
            printerror!("Could not open output file. {err}.");
            return Err(());
        }
    });

    printinfo!("Translating {input_path:?} -> {output_path:?}");
    if do_dry_run {
        dry_run(input_data, output_data);
        return Ok(());
    }

    let mut overwritten_input_data: Vec<String> = Vec::with_capacity(input_data.len());

    // Add header to csv file.
    let output = if use_markdown_output {
        printinfo!(be_verbose, "Using markdown style header.");
        Requirement::get_md_header()
    } else {
        printinfo!(be_verbose, "Using csv style header.");
        Requirement::get_csv_header()
    };
    let res = spreadsheet_writer.write(&output
        .chars()
        .map(|x| x as u8)
        .collect::<Vec<u8>>());
    if let Err(err) = res {
        printerror!("Error while writing spreadsheet header. {err}");
    }

    let mut category = String::new();

    printinfo!(be_verbose, "\nWriting to {output_path:?}");

    // Iterate over input data.
    // If $key exists in both input and output file, update status.
    for mut req in input_data {
        printinfo!(be_verbose, "READ TXT: {}", req.to_text_format());

        // Update status info, if data was find in spreadsheet.
        if let Some(val) = output_data.get(&req.hash) {
            printinfo!(be_verbose, "COMPARE TO CSV: {}", val.to_csv_format().trim_end());
            // If csv is provided, it is assumed to take authority over txt.
            req.copy_status(&val, be_verbose);
        }
        if *req.category != category {
            category = req.category.to_string();
            let long_cat = match categories.get(&category) {
                Some(cat) => cat,
                None => {
                    printerror!("Could not get category. Key={category}");
                    return Err(());
                }
            };
            overwritten_input_data.push(long_cat.to_string());
        }

        // Save updated and reformatted data to overwrite input file later.
        overwritten_input_data.push(req.to_text_format());
        printinfo!(be_verbose, "WRITE TXT -> CSV: {} ", req.to_csv_format());

        let output = if use_markdown_output {
            req.to_md_format()
        } else {
            req.to_csv_format()
        };
        let res = spreadsheet_writer.write(&output
            .chars()
            .map(|x| x as u8)
            .collect::<Vec<u8>>());
        if let Err(err) = res {
            printerror!("Error while writing spreadsheet. {err}");
        }
    }
    if let Err(err) = spreadsheet_writer.flush() {
        printerror!("Error while writing spreadsheet. {err}");
    }

    printinfo!(be_verbose, "\nOverwriting {input_path:?}");

    // Writer to original requirements file.
    let mut requirements_writer = BufWriter::new(match File::create(&input_path) {
        Ok(writer) => writer,
        Err(err) => {
            printerror!("Could not open input file. {err}.");
            return Err(());
        }
    });

    if overwrite_original_file {
        let _ = requirements_writer.write_all(&overwritten_input_data
            .join("\n")
            .chars()
            .map(|x| x as u8)
            .collect::<Vec<u8>>());
        if let Err(err) = requirements_writer.flush() {
            printerror!("Error while overwriting requirements file. {err}");
        }
    }

    return Ok(());
}

fn print_help() {
    println!("Takes a text file containing a list of requirements and translates them into a spreadsheet.");
    println!("ody [options] requirements_file [spreadsheet]");
    println!("If [spreadsheet] is a valid csv file, it is treated as a previous version and odysseus will attempt to preserve its data.");
    println!("ody will auto generate a unique id for each requirement, by taking a hash of its text contents. This id will be appended to each list item wrapped in '(@<hash>)'. However, if odysseus finds a value of this form in the input file, it will use that instead.");
    println!("\n\nOptions:");
    println!("-h | --help\t\tShow this menu.\n-o | --output path\tWrite spreadsheet to $path.\n-w | --no-overwrite\tDon't overwrite original requirements file.\n-n | --dry-run\t\tRun command without writing to fs.\n-m | --markdown\t\tSave output as markdown style table instead of csv.");
}

fn dry_run(input_data: Vec<Requirement>, output_data: HashMap<String, Requirement>) {
    println!();
    for mut req in input_data {
        println!("READ TXT: {}", req.to_text_format());

        // Update status info, if data was find in spreadsheet.
        if let Some(val) = output_data.get(&req.hash) {
            println!("COMPARE TO CSV: {}", val.to_csv_format().trim_end());
            println!("EDIT STATUS: {} -> {}", req.status, val.status);
            req.status = val.status;
        }
        println!("WRITE TXT -> CSV: {} ", req.to_csv_format());
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let input_path = PathBuf::from("tests/test_compare.txt");
        let output_path = PathBuf::from("tests/test_compare.csv");
        printinfo!("Translating {input_path:?} -> {output_path:?}");

        let input_data = parse_requirements(&input_path, true).unwrap().0;
        let output_data = parse_spreadsheet(&output_path, true).unwrap();
        dry_run(input_data, output_data);
        // assert!(false);
        assert!(true);
    }
}
