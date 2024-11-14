mod requirements;
mod spreadsheet_reader;

use std::fs;
use std::path::PathBuf;

use mythos_core::cli::clean_cli_args;

fn main() {
    let input_file: PathBuf;
    let output_file: PathBuf;
    // Receive 1-2 args.
    let args = clean_cli_args();
    if args.len() >= 1 {
        if args[0] == "-h".to_string()  || args[0] == "--help".to_string() {
            print_help();
            return;
        }
        input_file = PathBuf::from(args[0].to_owned());
    }
    if args.len() == 2 {
        output_file = PathBuf::from(args[1].to_owned());
    }

    // Read input file's contents.
    // Parse csv data, iterating over each line.
    // Try generating a unique id for each item. If item ends with (@<hash>), then use <hash> as
    // id. Otherwise, take a hash of text contents and use that instead.
}

fn print_help() {
    println!("Takes a text file containing a list of requirements and translates them into a spreadsheet.");
    println!("ody input-file [output-file]");
    println!("If [output-file] is a valid csv file, it is treated as a previous version and odysseus will attempt to preserve its data.");
    println!("ody will auto generate a unique id for each requirement, by taking a hash of its text contents. This id will be appended to each list item wrapped in '(@<hash>)'. However, if odysseus finds a value of this form in the input file, it will use that instead.");
}
