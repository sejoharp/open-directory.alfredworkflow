extern crate json;

use std::env;
use std::fs;
use std::fs::DirEntry;
use std::ops::Neg;
use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgMatches, Command};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use itertools::Itertools;
use powerpack::Item;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Directory {
    name: String,
    path: String,
}

impl Directory {
    pub fn from_pathbuf(path: &PathBuf) -> Directory {
        let name = path
            .file_name()
            .map_or("", |file_name| file_name.to_str().unwrap_or_default());
        let path = path
            .as_path()
            .to_str().unwrap_or_default();
        Directory {
            name: String::from(name),
            path: String::from(path),
        }
    }

    pub fn to_item(&self, binary_to_execute: String) -> Item {
        Item::new(self.name.to_string())
            .subtitle(format!("execute → {} {}", binary_to_execute, self.path.to_owned()))
            .arg(self.path.to_owned())
    }

    pub fn calculate_matching_score(&self, query: String) -> i64 {
        let matcher = SkimMatcherV2::default();
        return matcher
            .fuzzy_match(&self.name, &query)
            .get_or_insert(0)
            .to_owned()
            .neg();
    }
}


fn sort_and_filter_matching_directories(directories: Vec<Directory>, query: String) -> Vec<Directory> {
    return directories
        .iter()
        .sorted_by_key(|directory| directory.calculate_matching_score(query.to_owned()))
        .filter(|directory| directory.calculate_matching_score(query.to_owned()) < 0)
        .map(|directory| directory.to_owned())
        .collect();
}

fn to_items(directories: Vec<Directory>, query: String, binary_to_execute: String) -> Vec<Item> {
    let matched_directories: Vec<Item> = sort_and_filter_matching_directories(directories, query.clone())
        .iter()
        .map(|directory| directory.to_item(binary_to_execute.to_owned()))
        .collect();
    return if matched_directories.is_empty() {
        vec![default(query)]
    } else {
        matched_directories
    };
}

fn read_directories(full_path: String) -> Vec<PathBuf> {
    let directories_pathes = fs::read_dir(full_path)
        .expect("unable to read sub DIRECTORY_PATH")
        .filter_map(|entry| entry.ok())
        .filter(|entry| is_dir(entry))
        .map(|directory| directory.path());
    return directories_pathes.collect::<Vec<PathBuf>>();
}

fn is_dir(entry: &DirEntry) -> bool {
    fs::metadata(entry.path())
        .unwrap()
        .is_dir()
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

fn main() -> Result<()> {
    let matches = cli();

    match matches.subcommand() {
        Some(("search", sub_matches)) => search_for_directories(sub_matches),
        Some(("open", sub_matches)) => open_directory(sub_matches),
        _ => unreachable!(),
    }
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

fn search_for_directories(arguments: &ArgMatches) -> Result<()> {
    let directory_path = env::var("DIRECTORY_PATH")
        .expect("DIRECTORY_PATH not set");
    let binary_to_execute = env::var("BINARY_TO_EXECUTE")
        .expect("BINARY_TO_EXECUTE not set");
    let directory_pathes = read_directories(directory_path);
    let directories: Vec<Directory> = directory_pathes
        .iter()
        .map(Directory::from_pathbuf)
        .collect();

    let pattern = arguments.get_one::<String>("pattern");
    let items: Vec<Item> = match pattern.map(|pattern| pattern.as_str().trim()) {
        None | Some("") => vec![default(String::from(""))],
        Some(pattern) => to_items(directories, String::from(pattern), binary_to_execute),
    };
    powerpack::output(items)?;
    Ok(())
}

fn default(query: String) -> Item {
    let message = if query.is_empty() {
        "please add a search pattern".to_string()
    } else {
        format!("nothing found for {}, try to reformulate your search", query)
    };
    Item::new(message)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{Directory, read_directories, sort_and_filter_matching_directories};

    #[test]
    fn transforms_pathbuf_to_directory() {
        let path = PathBuf::from("tests/resources/target_dir/example_dir1");
        let expected = Directory {
            name: "example_dir1".to_string(),
            path: "tests/resources/target_dir/example_dir1".to_string(),
        };

        let actual = Directory::from_pathbuf(&path);

        assert_eq!(expected, actual);
    }

    #[test]
    fn transforms_pathbufs_to_directories() {
        let directory_pathes = read_directories(String::from("tests/resources/target_dir/"));
        let expected = vec![
            PathBuf::from("tests/resources/target_dir/example_dir1"),
            PathBuf::from("tests/resources/target_dir/example_dir2")];
        assert_eq!(expected, directory_pathes);
    }

    #[test]
    fn findes_two_dirs() {
        let directory_pathes = read_directories(String::from("tests/resources/target_dir/"));
        let expected = vec![
            PathBuf::from("tests/resources/target_dir/example_dir1"),
            PathBuf::from("tests/resources/target_dir/example_dir2")];
        assert_eq!(expected, directory_pathes);
    }

    #[test]
    fn does_not_matches_the_query() {
        let directory = Directory {
            name: String::from("Dashboard"),
            path: String::from("http://www.test.blub"),
        };

        let score = directory.calculate_matching_score("z".to_string());

        assert_eq!(score, 0);
    }

    #[test]
    fn matches_the_query() {
        let directory = Directory {
            name: String::from("Dashboard"),
            path: String::from("http://www.test.blub"),
        };

        let score = directory.calculate_matching_score("d".to_string());

        assert_eq!(score, -29);
    }


    #[test]
    fn sorts_and_keep_matchting_directories() {
        let directory1 = Directory {
            name: String::from("dashboard"),
            path: String::from("/test/dashboard"),
        };
        let directory2 = Directory {
            name: String::from("bookmarks"),
            path: String::from("/test/boomarks"),
        };
        let directories = vec![directory1.clone(), directory2.clone()];
        let expected_directories = vec![directory1.clone(), directory2.clone()];

        let matching_directories = sort_and_filter_matching_directories(directories, "o".to_owned());

        assert_eq!(matching_directories, expected_directories);
    }

    #[test]
    fn removes_not_matchting_directories() {
        let directory1 = Directory {
            name: String::from("Dashboard"),
            path: String::from("http://www.test.blub"),
        };
        let directory2 = Directory {
            name: String::from("Bookmarks"),
            path: String::from("http://www.bookmarks.blub"),
        };
        let directories = vec![directory1.clone(), directory2.clone()];
        let expected_directories = vec![directory1.clone()];

        let matching_directories = sort_and_filter_matching_directories(directories, "d".to_owned());

        assert_eq!(matching_directories, expected_directories);
    }
}
