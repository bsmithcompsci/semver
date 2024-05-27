use std::{collections::HashMap, fs::File, hash::Hash, io};

use clap::Parser;
use git2::Commit;
use log::{debug, error, info};

#[derive(serde::Deserialize, Debug)]
enum CommitType
{
    MAJOR,
    MINOR,
    PATCH,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json_input: Option<String>,

    #[arg(short, long, default_value = ".")]
    repository: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct SemverDataTaggingRepository
{
    enabled: bool,
    token_env: Option<String>,
}
#[derive(serde::Deserialize, Debug)]
struct SemverDataTagging
{
    patterns: String,
    supported_repositories: HashMap<String, SemverDataTaggingRepository>,
}

#[derive(serde::Deserialize, Debug)]
struct SemverDataBranch
{
    name: String,
    prerelease: Option<bool>,
    increment: Option<Vec<String>>
}

#[derive(serde::Deserialize, Debug)]
struct SemverDataCommits
{
    default: String,
    caseSensitive: bool,
    map: HashMap<String, Vec<String>>
}

#[derive(serde::Deserialize, Debug)]
struct SemverData {
    tagging: SemverDataTagging,
    branches: Vec<SemverDataBranch>,
    commits: SemverDataCommits
}

fn main() {
    // Initialize the logger, while in debug mode, log everything; otherwise, log only errors, warnings and info.
    if cfg!(debug_assertions) 
    {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::max())
            .init();
    } 
    else 
    {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Parse the command line arguments
    let args = Args::parse();

    // Check if the JSON file path is provided
    let json_file: String = if let Some(json_input) = args.json_input {
        if json_input.is_empty() {
            error!("Json File Path is empty!");
            return;
        }

        json_input.clone()
    } else {
        String::from(".semver.json")
    };

    // Check if the file exists.
    if !std::path::Path::new(&json_file).exists() {
        error!("Json File: `{}` does not exist!", json_file);
        return;
    }

    // Parse the JSON file with Serde
    let file = File::open(&json_file).unwrap();
    let reader = io::BufReader::new(file);
    let data: serde_json::Value = serde_json::from_reader(reader).unwrap();

    // Parse the JSON data into SemverData
    let semver_data: SemverData = serde_json::from_value(data).unwrap();

    info!("{:?}", semver_data);

    // Check if the repository is provided
    let repository_base_path: String = if let Some(repository_base_path) = args.repository {
        if repository_base_path.is_empty() {
            error!("Repository is empty!");
            return;
        }

        repository_base_path.clone()
    } else {
        String::from(".")
    };

    let repository = git2::Repository::open(repository_base_path).unwrap();
    if repository.is_bare()
    {
        error!("Repository is bare!");
        return;
    }

    // Get Current Branch
    let head = repository.head().unwrap();
    let branch = head.shorthand().unwrap();
    info!("Selected Branch: {}", branch);

    // Get all Tags
    let tags = repository.tag_names(Some("*")).unwrap();
    for tag in tags.iter() {
        info!("Tag: {}", tag.unwrap());
    }

    // Get all Commits
    let mut revwalk = repository.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let commits: Vec<git2::Commit> = revwalk
        .map(|id| repository.find_commit(id.unwrap()).unwrap())
        .collect();

    info!("Commits: {}", commits.len());
    // Print all commits
    for commit in commits.iter() {
        let commit_id = commit.id();
        let commit_message = commit.message().unwrap();
        let commit_author = commit.author();

        // First word of the commit message
        let first_word = commit_message.split_whitespace().next().unwrap();

        // Check if the first word is in the map
        let mut commit_type : CommitType = CommitType::PATCH;
        for (key, value) in semver_data.commits.map.iter() {
            if value.iter().any(|x| 
                (semver_data.commits.caseSensitive && x.contains(&first_word)) || 
                (semver_data.commits.caseSensitive && x.to_lowercase().contains(&first_word.to_lowercase()))
            ) {
                // Parse the Key to the Commit Type, default is PATCH.
                commit_type = match key.to_uppercase().as_str() {
                    "MAJOR" => CommitType::MAJOR,
                    "MINOR" => CommitType::MINOR,
                    "PATCH" => CommitType::PATCH,
                    _ => CommitType::PATCH,
                };
                break;
            }
        }

        info!(
            "Commit: {} - {} - {} [{:?}]",
            commit_id, commit_author.name().unwrap(), commit_message, commit_type
        );
    }
}
