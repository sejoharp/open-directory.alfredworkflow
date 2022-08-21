extern crate json;

use std::env;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgMatches, Command};
use powerpack::Item;

use directory::Directory;

mod directory;


fn read_directories(full_pathes: String) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = vec![];
    for full_path in split_at_comma(full_pathes) {
        let mut directories_pathes = fs::read_dir(full_path)
            .expect("unable to read sub DIRECTORY_PATH")
            .filter_map(|entry| entry.ok())
            .filter(|entry| is_dir(entry))
            .map(|directory| directory.path())
            .collect::<Vec<PathBuf>>();
        result.append(&mut directories_pathes)
    }
    return result;
}

fn is_dir(entry: &DirEntry) -> bool {
    fs::metadata(entry.path())
        .unwrap()
        .is_dir()
}


fn default(query: String) -> Item {
    let message = if query.is_empty() {
        "please add a search pattern".to_string()
    } else {
        format!("nothing found for {}, try to reformulate your search", query)
    };
    Item::new(message)
}

fn cli() -> ArgMatches {
    let app = Command::new("open-directory-alfred-workflow")
        .subcommand_required(true)
        .subcommand(
            Command::new("search").arg(
                clap::arg!(--pattern <PATTERN>)
                    .required(true)
                    .value_parser(clap::value_parser!(String)),
            ),
        )
        .subcommand(
            Command::new("open").arg(
                clap::arg!(--path <PATH>)
                    .required(true)
                    .value_parser(clap::value_parser!(String)),
            )
        );

    return app.get_matches();
}

fn open_directory(arguments: &ArgMatches) -> Result<()> {
    let binary_to_execute = env::var("BINARY_TO_EXECUTE")
        .expect("BINARY_TO_EXECUTE not set");
    let path = arguments
        .get_one::<String>("path")
        .expect("PATH cannot be empty.");

    std::process::Command::new(binary_to_execute)
        .arg(path)
        .output()
        .expect("failed to execute process");
    Ok(())
}

fn split_at_comma(directories: String) -> Vec<String> {
    directories
        .split(",")
        .map(|directory| directory.trim())
        .map(String::from)
        .collect()
}

fn search_for_directories(arguments: &ArgMatches) -> Result<()> {
    let directory_pathes = env::var("DIRECTORY_PATH")
        .expect("DIRECTORY_PATH not set");
    let binary_to_execute = env::var("BINARY_TO_EXECUTE")
        .expect("BINARY_TO_EXECUTE not set");

    let directory_pathes = read_directories(directory_pathes);
    let directories: Vec<Directory> = directory_pathes
        .iter()
        .map(Directory::from_pathbuf)
        .collect();

    let pattern = arguments.get_one::<String>("pattern");
    let items: Vec<Item> = match pattern.map(|pattern| pattern.as_str().trim()) {
        None | Some("") => vec![default(String::from(""))],
        Some(pattern) => Directory::transform_to_items(directories, String::from(pattern), binary_to_execute),
    };
    powerpack::output(items)?;
    Ok(())
}

fn main() -> Result<()> {
    let matches = cli();

    match matches.subcommand() {
        Some(("search", sub_matches)) => search_for_directories(sub_matches),
        Some(("open", sub_matches)) => open_directory(sub_matches),
        _ => unreachable!(),
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{read_directories, split_at_comma};

    #[test]
    fn findes_two_dirs() {
        let directory_pathes = read_directories(String::from("tests/resources/target_dir/"));
        let expected = vec![
            PathBuf::from("tests/resources/target_dir/example_dir1"),
            PathBuf::from("tests/resources/target_dir/example_dir2")];
        assert_eq!(expected, directory_pathes);
    }

    #[test]
    fn splits_into_two_string() {
        let expected = vec![
            "tests/resources/target_dir/example_dir1".to_string(),
            "tests/resources/target_dir/example_dir2".to_string()];

        let directory_strings = split_at_comma("tests/resources/target_dir/example_dir1, tests/resources/target_dir/example_dir2".to_string());

        assert_eq!(expected, directory_strings)
    }

    #[test]
    fn splits_into_one_string() {
        let expected = vec![
            "tests/resources/target_dir/example_dir2".to_string()];

        let directory_strings = split_at_comma("tests/resources/target_dir/example_dir2".to_string());

        assert_eq!(expected, directory_strings)
    }
}
