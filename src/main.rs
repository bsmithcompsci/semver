use std::{collections::HashMap, fs::File, hash::Hash, io};

use clap::Parser;
use git2::{Commit, Oid, Tag, Version};
use log::{debug, error, info};

#[derive(serde::Deserialize, Debug)]
enum CommitType
{
    MAJOR,
    MINOR,
    PATCH,
}

#[derive(serde::Deserialize, Debug)]
struct SemanticVersion
{
    major: u32,
    minor: u32,
    patch: u32,

    // Prefix & Suffix
    prefix: Option<String>,
    suffix: Option<String>,
}

impl SemanticVersion
{
    // Ctor
    fn new() -> SemanticVersion
    {
        SemanticVersion { major: 1, minor: 0, patch: 0, prefix: None, suffix: None }
    }

    // Increment
    fn increment(&mut self, commit_type: &CommitType)
    {
        match commit_type
        {
            CommitType::MAJOR => self.major += 1,
            CommitType::MINOR => self.minor += 1,
            CommitType::PATCH => self.patch += 1,
        }
    }
    // Decrement
    fn decrement(&mut self, commit_type: &CommitType)
    {
        match commit_type
        {
            CommitType::MAJOR => self.major -= 1,
            CommitType::MINOR => self.minor -= 1,
            CommitType::PATCH => self.patch -= 1,
        }
    }

    // ToString
    fn to_string(&self) -> String
    {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        // [prefix-]x.x.x
        if let Some(prefix) = &self.prefix
        {
            version = format!("{}-{}", prefix, version);
        }
        // [prefix-]x.x.x[-suffix]
        if let Some(suffix) = &self.suffix
        {
            version = format!("{}-{}", version, suffix);
        }
        version
    }

    // Parse
    fn parse(version: &str) -> SemanticVersion
    {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;

        let parts = version.split('-').collect::<Vec<&str>>();
        let version_part_index = if parts.len() > 1 { 1 } else { 0 };

        let version_parts = parts[version_part_index].split('.').collect::<Vec<&str>>();
        if version_parts.len() > 0
        {
            major = version_parts[0].parse::<u32>().unwrap();
        }
        if version_parts.len() > 1
        {
            minor = version_parts[1].parse::<u32>().unwrap();
        }
        if version_parts.len() > 2
        {
            patch = version_parts[2].parse::<u32>().unwrap();
        }

        let prefix = if parts.len() > 1 { Some(parts[0].to_string()) } else { None };
        let suffix = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };
        
        SemanticVersion { major, minor, patch, prefix, suffix }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json_input: Option<String>,

    #[arg(short, long, default_value = ".")]
    repository: Option<String>,

    #[arg(long, action)]
    release: bool
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
    release: String,
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
    info!("Read Semantic Version Data");
    // Check if the repository is provided
    let repository_base_path: String = if let Some(repository_base_path) = args.repository 
    {
        if repository_base_path.is_empty() 
        {
            error!("Repository is empty!");
            return;
        }

        repository_base_path.clone()
    } 
    else 
    {
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
    let mut commit_tags = HashMap::<Oid, Tag>::new();
    let tags = repository.tag_names(Some("*")).unwrap();
    for tag_name in tags.iter() 
    {
        let obj = repository.revparse_single(tag_name.unwrap()).unwrap();
        if let Some(tag) = obj.as_tag() 
        {
            // Now lets get the commit for the tag
            let commit = tag.target().unwrap().peel_to_commit().unwrap();
            commit_tags.insert(commit.id(), tag.clone());
        }
    }

    let mut version = SemanticVersion::new();
    let latest_tag = commit_tags.iter().next();
    if let Some((_, tag)) = latest_tag 
    {
        let tag_name = tag.name().unwrap();
        info!("Latest Tag: {}", tag_name);
        version = SemanticVersion::parse(tag_name);
    }

    // Get all Commits
    let mut revwalk = repository.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let commits: Vec<git2::Commit> = revwalk
        .map(|id| repository.find_commit(id.unwrap()).unwrap())
        .collect();

    info!("Commits: {}", commits.len());

    // Print all commits
    for commit in commits.iter() 
    {
        let mut should_release = args.release;

        let commit_id = commit.id();
        let commit_message = commit.message().unwrap();
        let commit_author = commit.author();

        // Check if the commit is tagged
        let tag: Option<Tag> = if commit_tags.contains_key(&commit_id) 
        {
            let tags = commit_tags.clone();
            let tag = tags.get(&commit_id).unwrap();
            Some(tag.clone())
        } 
        else 
        {
            None
        };

        // Do not continue, if the commit is tagged.
        if tag.is_some() 
        {
            break;
        }

        // First word of the commit message
        let first_word = commit_message.split_whitespace().next().unwrap();

        // Check if the first word is in the map
        let mut commit_type : CommitType = CommitType::PATCH;
        for (key, value) in semver_data.commits.map.iter() 
        {
            for value in value.iter() 
            {
                if (semver_data.commits.caseSensitive && first_word == value) || (!semver_data.commits.caseSensitive && first_word.contains(value)) 
                {
                    // Parse the Key to the Commit Type, default is PATCH.
                    commit_type = match key.to_uppercase().as_str() 
                    {
                        "MAJOR" => CommitType::MAJOR,
                        "MINOR" => CommitType::MINOR,
                        "PATCH" => CommitType::PATCH,
                        _ => CommitType::PATCH,
                    };
                    break;
                }
            }
        }

        // Trigger Release.
        if first_word.contains(format!("({})", semver_data.commits.release).as_str()) 
        {
            should_release = true;
        }

        if should_release 
        {
            // Increment the version
            version.increment(&commit_type);
            
            // Tag the commit
            let tag_name = version.to_string();
            let tag_message = format!("Release: {}", tag_name);
            let tag_oid = repository.tag(tag_name.as_str(), &commit.as_object(), &commit_author, tag_message.as_str(), false).unwrap();
            let tag = repository.find_tag(tag_oid).unwrap();
            commit_tags.insert(commit_id, tag);
        }

        info!(
            "Commit: [{:?}] {}{} - {} - {}",
            commit_type, if tag.is_some() { format!("[TAGGED: {}] ", tag.unwrap().name().unwrap()) } else { "".to_string() }, commit_id, commit_author.name().unwrap(), commit_message
        );
    }
}
