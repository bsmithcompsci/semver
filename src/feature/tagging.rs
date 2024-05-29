use log::{debug, info, error};

use crate::libs::{release::{Release, ReleaseContributor, ReleaseType}, version::{self, SemanticVersion}};

pub fn tag(args: crate::Args, release: &Release, commit: &git2::Commit, repository: &git2::Repository) -> Option<git2::Oid>
{
    let app_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let app_repository_url = std::env::var("CARGO_PKG_REPOSITORY").unwrap();
    let commit_author = commit.author();

    // Tag the commit
    let tag_name = release.version.to_string();
    // Build the tag message
    let mut tag_message = String::new();
    {
        tag_message.push_str(format!("# {} {}", if release.tag == ReleaseType::Release { "Release" } else { "Pre-Release" }, tag_name).as_str());
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

        tag_message.push_str("---\n");

        tag_message.push_str(format!("Generated by: [{}]({})", app_name, app_repository_url).as_str());
    }

    debug!("Message:\n{}", tag_message.as_str());
    
    if args.dry_run
    {
        info!("Dry Run: Tagging: {} for {}", tag_name.as_str(), commit.id());
        return Some(git2::Oid::zero())
    }

    debug!("Tagging: {} for {:?}", tag_name.as_str(), commit);

    let tag_oid = repository.tag(tag_name.as_str(), &commit.as_object(), &commit_author, tag_message.as_str(), true).unwrap();

    // Callbacks
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(crate::git_credentials_callback);

    // Push Tags.
    let mut remote = repository.find_remote("origin").unwrap();
    if let Err(error) = remote.push(&[format!("refs/tags/{}", tag_name.as_str()).to_string()], Some(git2::PushOptions::new().remote_callbacks(callbacks)))
    {
        error!("Failed to push Tag: {} for {}\n\t{:?}", tag_name.as_str(), commit.id(), error);
        // Cleanup Error.
        repository.tag_delete(&tag_name.as_str()).unwrap();
        if args.exit_on_error
        {
            std::process::exit(1);
        }
    }
    else
    {
        info!("Pushed Tag: {} for {}", tag_name.as_str(), commit.id());
    }

    Some(tag_oid)
}

#[test]
fn test_tagging()
{
    use crate::libs::release::Release;
    use crate::libs::release::ReleaseType;

    let _ = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    {
        let home = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".to_string()));
        let var = Some(format!("{}/.ssh/Github", home));
        std::env::set_var("GIT_SSH_KEY", var.clone().unwrap());
        debug!("Git Credentials Authenticated: {}", var.clone().unwrap());
    }

    let rand_major = rand::random::<u8>();

    let mut version = SemanticVersion::new();
    version.increment_by(&version::CommitType::MAJOR, rand_major as u32);

    let repository = git2::Repository::open(".").unwrap();
    let commit = repository.head().unwrap().peel_to_commit().unwrap();
    let release = Release {
        version: version.clone(),
        tag: ReleaseType::Release,
        commit: commit.id(),
        majors: vec!["Major Change".to_string()],
        minors: vec!["Minor Change".to_string()],
        patches: vec!["Patch Change".to_string()],
        contributors: vec![ReleaseContributor { name: "Name".to_string(), email: "Test@email.com".to_string() }],
    };

    let args = crate::Args::default();

    let tag_oid = tag(args, &release, &commit, &repository);

    assert!(tag_oid.is_some());
    assert!(tag_oid.unwrap().is_zero() == false);

    let tag_name = release.version.to_string();

    // Cleanup when in test.
    repository.tag_delete(&tag_name.as_str()).unwrap();

    // Callbacks
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(crate::git_credentials_callback);
    
    // Push Tags.
    let mut remote = repository.find_remote("origin").unwrap();
    if let Err(error) = remote.push(&[format!("refs/tags/{}", tag_name.as_str()).to_string()], Some(git2::PushOptions::new().remote_callbacks(callbacks)))
    {
        error!("Failed to push Tag: {} for {}\n\t{:?}", tag_name.as_str(), commit.id(), error);
    }
    else
    {
        info!("Pushed Tag: {} for {}", tag_name.as_str(), commit.id());
    }
}