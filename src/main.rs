use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use glob::glob;
use kuchiki::{parse_html, traits::TendrilSink};
use regex::Regex;

fn main() {
    let url_pattern = r"https?://([\w\-]+\.)+[\w\-]+(/[\w\-./?%&=]*)?";
    let url_regex = Regex::new(url_pattern).unwrap();

    let files = glob("./**/*.html")
        .expect("fialed to read glob")
        .map(|f| f.unwrap())
        .collect::<Vec<_>>();

    files
        .iter()
        .for_each(|f| parse_file(f, &url_regex, &files).unwrap());
}

fn get_local_path(file: &str) -> Option<String> {
    // Get the current working directory
    let current_dir = env::current_dir().ok()?;
    let file_path = Path::new(file);

    // Check if the file path is relative to the current directory
    if let Ok(relative_path) = file_path.strip_prefix(&current_dir) {
        Some(relative_path.to_string_lossy().into_owned())
    } else {
        println!("{} vs {}", current_dir.display(), file_path.display());
        None
    }
}

fn parse_file(file: &Path, url_regex: &Regex, files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let file_str = fs::read_to_string(file)?;
    let document = parse_html().from_utf8().one(file_str.as_bytes());

    let tags = document.select("a").unwrap();
    for t in tags {
        let a_tag = t.as_node();

        if let Some(attributes) = a_tag
            .as_element()
            .unwrap()
            .attributes
            .borrow_mut()
            .get_mut("href")
        {
            if !url_regex.is_match(attributes) {
                *attributes = format!(
                    "/{}",
                    get_local_path(
                        &fs::canonicalize(find_matching_file(files, attributes).unwrap())
                            .unwrap()
                            .display()
                            .to_string()
                    )
                    .unwrap()
                )
            }
        }
    }

    let mut output = File::create(file)?;
    write!(output, "{}", document.to_string())?;

    Ok(())
}

fn find_matching_file(paths: &[PathBuf], part: &str) -> Option<PathBuf> {
    paths
        .iter()
        .find(|path| path.to_string_lossy().contains(part))
        .cloned()
}
