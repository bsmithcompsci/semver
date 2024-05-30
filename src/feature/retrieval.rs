use std::collections::HashMap;

use log::{debug, error, info, warn};

use crate::{libs::{release::{Release, ReleaseContributor, ReleaseType}, version::{CommitType, SemanticVersion}}, SemverData};

pub fn get(args: crate::Args, semver_data: &SemverData, repository: &git2::Repository) -> Vec<Release>
{
    // Get Current Branch
    let head = repository.head().unwrap();
    let branch = head.shorthand().unwrap();
    info!("Selected Branch: {}", branch);

    // Get all Tags
    let mut commit_tags = HashMap::<git2::Oid, git2::Tag>::new();
    let tags = repository.tag_names(None).unwrap();
    
    // Sort Tags.
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

    // Print all Tags
    for (commit_id, tag) in commit_tags.iter() 
    {
        debug!("Tag: {} - {}", commit_id, tag.name().unwrap());
    }

    // Get all Commits
    let mut revwalk = repository.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let mut commits: Vec<git2::Commit> = revwalk
        .map(|id| repository.find_commit(id.unwrap()).unwrap())
        .collect();

    // Print all Commits
    for commit in commits.iter() 
    {
        debug!("Commit: {} - {}", commit.id(), commit.message().unwrap());
    }

    commits.reverse();

    // Cleanup commits that are within a tag.
    let mut version = SemanticVersion::new();
    {
        let last_commit_index = {
            let mut commit_tag_index = 0;
            for (index, commit) in commits.iter().enumerate()
            {
                if commit_tags.contains_key(&commit.id()) 
                {
                    commit_tag_index = index + 1;
                }
            }
            commit_tag_index
        };

        // Get the last commit that is not tagged.
        let last_commit = commits[last_commit_index-1].clone();
        if let Some(tag) = commit_tags.get(&last_commit.id())
        {
            let tag_version = tag.name().unwrap();
            debug!("Last Tag: {} - {}", last_commit.id(), tag_version);
            version = SemanticVersion::parse(tag_version);
        }

        commits = commits[last_commit_index..].to_vec();
    }
    let version = version; // De-mut the variable.

    info!("Commits: {}", commits.len());

    // Store Data about the current Version Release.
    //  Later on, we will catchup with the commits and create a new release.
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
        let mut release_type;
        if commits.last().unwrap().id() == commit.id()
        {
            if args.force_release
            {
                release_type = ReleaseType::Release;
            }
            else if args.force_prerelease
            {
                release_type = ReleaseType::PreRelease;
            }
            else
            {
                release_type = ReleaseType::None;
            }
        }
        else
        {
            release_type = ReleaseType::None;
        }

        let commit_id = commit.id();
        let commit_message = commit.message().unwrap();
        let commit_author = commit.author();

        // Check if the commit is tagged
        let tag: Option<git2::Tag> = if commit_tags.contains_key(&commit_id) 
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
        let regex_str = regex::Regex::new(r#"^([a-zA-Z]+\s*)+(\([a-zA-Z]+\)|)(!?):"#).unwrap();
        // Check if the commit message follows the format.
        let captures = regex_str.captures(commit_message);
        
        let is_major;
        let follows_format: bool;
        let first_word;
        if let Some(captures) = captures
        {
            
            first_word = captures.get(0).unwrap().as_str();
            follows_format = true;

            is_major = captures.len() > 3 && captures.get(3).unwrap().as_str() == "!";
        }
        else
        {
            first_word = commit_message.split_whitespace().next().unwrap();
            follows_format = false;
            is_major = false;
        }

        // Check if the first word is in the map
        let mut skip = false;
        let mut commit_type : CommitType = CommitType::Patch;
        for (key, value) in semver_data.commits.map.iter() 
        {
            for value in value.iter() 
            {
                if (semver_data.commits.case_sensitive && first_word == value) || (!semver_data.commits.case_sensitive && first_word.contains(value)) 
                {
                    // Parse the Key to the Commit Type, default is PATCH.
                    commit_type = match key.to_uppercase().as_str() 
                    {
                        "MAJOR" => CommitType::Major,
                        "MINOR" => CommitType::Minor,
                        "PATCH" => CommitType::Patch,
                        _ => match semver_data.commits.default.to_uppercase().as_str() 
                        {
                            "MAJOR" => CommitType::Major,
                            "MINOR" => CommitType::Minor,
                            "PATCH" => CommitType::Patch,
                            _ => CommitType::Patch,
                        }
                    };
                    break;
                }
            }

            // Check if the commit message follows the format.
            if !follows_format
            {
                if args.skip_non_formatted
                {
                    warn!("Commit: [NON-FORMATTED] {} - {} - {}", commit_id, commit_author.name().unwrap(), commit_message);
                    skip = true;
                    break;
                }
                else
                {
                    error!("Commit: [ERROR: NON-FORMATTED] {} - {} - {}", commit_id, commit_author.name().unwrap(), commit_message);
                    if args.exit_on_error
                    {
                        std::process::exit(1);
                    }
                }
            }
        }

        if skip
        {
            continue;
        }

        if is_major
        {
            commit_type = CommitType::Major;
        }

        // Trigger Release.
        if semver_data.commits.release.iter().any(|x| first_word.contains(format!("({})", x).as_str())) || commit_type == CommitType::Major
        {
            release_type = ReleaseType::Release;
        }
        // Trigger Prerelease.
        if semver_data.commits.prerelease.iter().any(|x| first_word.contains(format!("({})", x).as_str()))
        {
            release_type = ReleaseType::PreRelease;
        }

        // Place Commit Messages into the correct array.
        let branch_rules = semver_data.branches.iter().find(
            |x| 
            regex::Regex::new(x.name.as_str()).unwrap().is_match(branch)
        );

        let mut can_increment = release_type != ReleaseType::None;
        if branch_rules.is_some()
        {
            let branch_rules = branch_rules.unwrap();
            if can_increment && branch_rules.prerelease.is_some() && branch_rules.prerelease.unwrap() {
                release_type = ReleaseType::PreRelease;
            }

            if branch_rules.increment.is_some()
            {
                can_increment = branch_rules.increment.clone().unwrap().contains(&format!("{:?}", commit_type).to_string());
            }
        }

        match commit_type 
        {
            CommitType::Major => release_majors.push(commit_message.to_string()),
            CommitType::Minor => release_minors.push(commit_message.to_string()),
            CommitType::Patch => release_patches.push(commit_message.to_string()),
        }

        let bad_emails = ["noreply."];
        // Verify that the author is not "banned."
        if !bad_emails.iter().any(|x| commit.author().email().unwrap().contains(x))
        {
            // Add the author to the contributors list, only if they are not already in the list.
            if !release_contributors.iter().any(|x| x.email.contains(commit_author.email().unwrap()))
            {
                let copy_author = git2::Signature::now(commit.author().name().unwrap(), commit.author().email().unwrap()).unwrap();
                release_contributors.push(ReleaseContributor { name: copy_author.name().unwrap().to_string(), email: copy_author.email().unwrap().to_string() });    
            }
        }
        
        if can_increment || args.always_increment
        {
            release_version.increment(&commit_type);
        }

        // We detected a new release, so we need to create a new release.
        if can_increment
        {
            // We need to close the current release.
            if current_release.is_some()
            {
                releases.push(current_release.clone().unwrap());
            }

            // Create a new release.
            //  Piece together the release data to catchup.
            let release = Release { 
                commit: commit_id,
                tag: release_type, 
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
            
            debug!("Switching Releases:\n\tOld - {:?}\n\tNew - {:?}", current_release, release.clone());
            current_release = Some(release);
        }

        info!(
            "Commit: [{:?}] {}{}{} - {} - {}",
            commit_type, 
            if tag.is_some() { format!("[TAGGED: {}] ", tag.unwrap().name().unwrap()) } else { "".to_string() }, 
            if can_increment { "[TAGGING] ".to_string() } else { "".to_string() }, 
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

    releases
}

#[test]
fn test_get()
{
    use crate::libs::version::SemanticVersion;
    use crate::libs::release::ReleaseType;
    use crate::SemverData;
    
    let _ = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    let args = crate::Args { dry_run: true, ..Default::default() };

    let semver_data = SemverData {
        branches: vec![],
        commits: crate::SemverDataCommits 
        {
            case_sensitive: false,
            default: "PATCH".to_string(),
            map: Default::default(),
            release: vec![],
            prerelease: vec![],
        
        },
        tagging: crate::SemverDataTagging {
            supported_repositories: Default::default(),
        },
    };
    let repository = git2::Repository::open(".").unwrap();

    let releases = get(args, &semver_data, &repository);

    if !releases.is_empty()
    {
        for release in releases.iter()
        {
            assert_ne!(release.version, SemanticVersion::new(), "Version is not set: {:?}", release);
            assert_ne!(release.tag, ReleaseType::None, "Tag is not set: {:?}", release);
            assert_ne!(release.contributors.len(), 0, "Contributors is not set: {:?}", release);
        }
    }
}
