use std::path::PathBuf;
use powerpack::Item;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::ops::Neg;
use itertools::Itertools;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Directory {
    pub(crate) name: String,
    pub(crate) path: String,
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
    pub(crate) fn sort_and_filter_matching_directories(directories: Vec<Directory>, query: String) -> Vec<Directory> {
        return directories
            .iter()
            .sorted_by_key(|directory| directory.calculate_matching_score(query.to_owned()))
            .filter(|directory| directory.calculate_matching_score(query.to_owned()) < 0)
            .map(|directory| directory.to_owned())
            .collect();
    }
    pub(crate) fn transform_to_items(directories: Vec<Directory>, query: String, binary_to_execute: String) -> Vec<Item> {
        let matched_directories: Vec<Item> = Directory::sort_and_filter_matching_directories(directories, query.clone())
            .iter()
            .map(|directory| directory.to_item(binary_to_execute.to_owned()))
            .collect();
        return if matched_directories.is_empty() {
            vec![crate::default(query)]
        } else {
            matched_directories
        };
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::Directory;

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

        let matching_directories = Directory::sort_and_filter_matching_directories(directories, "o".to_owned());

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

        let matching_directories = Directory::sort_and_filter_matching_directories(directories, "d".to_owned());

        assert_eq!(matching_directories, expected_directories);
    }
}
