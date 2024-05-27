use std::{collections::HashMap, fs::File, io};

use clap::Parser;
use git2::{Oid, Tag};
use log::{debug, error, info, warn};

mod lib;

use lib::version::SemanticVersion;
use lib::data::*;

use crate::lib::version::CommitType;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    json_input: Option<String>,

    #[arg(short, long, default_value = ".")]
    repository: Option<String>,

    #[arg(long, action)]
    release: bool,
    #[arg(long, action)]
    prerelease: bool,

    #[arg(long, action)]
    dry_run: bool,
}


#[derive(Debug, Clone)]
struct ReleaseContributor
{
    name: String,
    email: String,
}
#[derive(Debug, Clone)]
struct Release
{
    commit:         Oid,  
    release:        bool,
    version:        SemanticVersion,
    majors:         Vec<String>,
    minors:         Vec<String>,
    patches:        Vec<String>,
    contributors:   Vec<ReleaseContributor>,
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
    let mut commits: Vec<git2::Commit> = revwalk
        .map(|id| repository.find_commit(id.unwrap()).unwrap())
        .collect();

    commits.reverse();

    // Cleanup commits that are within a tag.
    {
        let last_commit_index = {
            let mut commit_tag_index = 0;
            let mut index = 0;
            for commit in commits.iter() 
            {
                if commit_tags.contains_key(&commit.id()) 
                {
                    commit_tag_index = index + 1;
                }
                index += 1;
            }
            commit_tag_index
        };
        commits = commits[last_commit_index..].to_vec();
    }

    info!("Commits: {}", commits.len());

    // Store Data about the current Version Release.
    let mut releases = Vec::<Release>::new();
    let mut current_release: Option<Release> = None;
    let mut release_version = version.clone();
    let mut release_majors = Vec::<String>::new();
    let mut release_minors = Vec::<String>::new();
    let mut release_patches = Vec::<String>::new();
    let mut release_contributors = Vec::<ReleaseContributor>::new();

    // Parse each commit and fill out information that is needed.
    for commit in commits.iter() 
    {
        let mut should_release = args.release && commits.last().unwrap().id() == commit.id();
        let mut should_prerelease = args.prerelease && commits.last().unwrap().id() == commit.id();

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
            warn!("Commit: [TAGGED: {}] {} - {} - {}", tag.unwrap().name().unwrap(), commit_id, commit_author.name().unwrap(), commit_message);
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
        if semver_data.commits.release.iter().any(|x| first_word.contains(format!("({})", x).as_str()))
        {
            should_release = true;
        }
        // Trigger Prerelease.
        if semver_data.commits.prerelease.iter().any(|x| first_word.contains(format!("({})", x).as_str()))
        {
            should_prerelease = true;
        }

        // Place Commit Messages into the correct array.
        match commit_type 
        {
            CommitType::MAJOR => release_majors.push(commit_message.to_string()),
            CommitType::MINOR => release_minors.push(commit_message.to_string()),
            CommitType::PATCH => release_patches.push(commit_message.to_string()),
        }

        // Add the author to the contributors list, only if they are not already in the list.
        if !release_contributors.iter().any(|x| x.email.contains(commit_author.email().unwrap())) 
        {
            let copy_author = git2::Signature::now(commit.author().name().unwrap(), commit.author().email().unwrap()).unwrap();
            release_contributors.push(ReleaseContributor { name: copy_author.name().unwrap().to_string(), email: copy_author.email().unwrap().to_string() });    
        }
        
        if should_release || should_prerelease
        {
            release_version.increment(&commit_type);
        }

        // We detected a new release, so we need to create a new release.
        if should_release || should_prerelease
        {
            // We need to close the current release.
            if current_release.is_some()
            {
                releases.push(current_release.clone().unwrap());
            }

            // Create a new release.
            let release = Release { 
                commit: commit_id,
                release: should_release, 
                version: release_version.clone(), 
                majors: release_majors.clone(), 
                minors: release_minors.clone(), 
                patches: release_patches.clone(), 
                contributors: release_contributors.clone() 
            };

            // Reset the release data.
            release_majors.clear();
            release_minors.clear();
            release_patches.clear();
            release_contributors.clear();
            
            current_release = Some(release);
        }

        info!(
            "Commit: [{:?}] {}{}{} - {} - {}",
            commit_type, 
            if tag.is_some() { format!("[TAGGED: {}] ", tag.unwrap().name().unwrap()) } else { "".to_string() }, 
            if should_release || should_prerelease { format!("[TAGGING] ") } else { "".to_string() }, 
            commit_id, 
            commit_author.name().unwrap(), 
            commit_message
        );
    }

    // Close the last release.
    if current_release.is_some()
    {
        releases.push(current_release.clone().unwrap());
    }

    info!("Releases: {}", releases.len());

    // Tag the commits
    for release in releases.iter()
    {
        let commit = repository.find_commit(release.commit).unwrap();
        let commit_author = commit.author();

        // Tag the commit
        let tag_name = release.version.to_string();
        // Build the tag message
        let mut tag_message = String::new();
        {
            tag_message.push_str(format!("# {} {}", if args.prerelease { "Prerelease" } else { "Release" }, tag_name).as_str());
            tag_message.push_str("\n\n");

            if release.majors.len() > 0 
            {
                tag_message.push_str("## Major Changes:\n");
                for patch in release.majors.iter() 
                {
                    tag_message.push_str(format!("* {}\n", patch).as_str());
                }
                tag_message.push_str("\n");
            }

            if release.minors.len() > 0 
            {
                tag_message.push_str("## Minor Changes:\n");
                for minor in release.minors.iter() 
                {
                    tag_message.push_str(format!("* {}\n", minor).as_str());
                }
                tag_message.push_str("\n");
            }

            if release.patches.len() > 0 
            {
                tag_message.push_str("## Patch Changes:\n");
                for major in release.patches.iter() 
                {
                    tag_message.push_str(format!("* {}\n", major).as_str());
                }
                tag_message.push_str("\n");
            }

            tag_message.push_str("## Credits:\n");
            for contributor in release.contributors.iter() 
            {
                tag_message.push_str(format!("* {} <{}>\n", contributor.name, contributor.email).as_str());
            }

            tag_message.push_str("\n");

            tag_message.push_str("---");

            tag_message.push_str("Generated by: Flex-Vers");
        }

        debug!("Message:\n{}", tag_message.as_str());
        
        if !args.dry_run
        {
            let tag_oid = repository.tag(tag_name.as_str(), &commit.as_object(), &commit_author, tag_message.as_str(), true).unwrap();
            let tag = repository.find_tag(tag_oid).unwrap();
            commit_tags.insert(release.commit, tag);

            info!("Tagged: {} for {}", tag_name.as_str(), commit.id());
        }
        else {
            info!("Dry Run: Tagging: {} for {}", tag_name.as_str(), commit.id());
        }
    }
}
