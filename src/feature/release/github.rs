use crate::libs::release::{Release, ReleaseType};

use log::{debug, error, info};

pub async fn create(args: crate::Args, release: &Release, tag_oid: &git2::Oid, repository: &git2::Repository) -> Result<(), &'static str>
{
    let token = std::env::var("GITHUB_TOKEN")
        .expect("GITHUB_TOKEN env variable is required to create a release on GitHub. This should be a Default Variable created by github.com.");

    if cfg!(debug_assertions)
    {
        let repository_env = repository.find_remote("origin")
            .expect("Failed to find the remote origin.")
            .url()
            .expect("Failed to get the remote origin URL.")
            .to_string();

        let (owner, repo) = repository_env.split_once("/").unwrap();
        let owner = owner.split_once("github.com:").unwrap().1;
        let repo = repo.replace(".git", "");

        let repository_env = format!("{}/{}", owner, repo);

        debug!("Loading Repository: {:?}", repository_env);

        std::env::set_var("GITHUB_REPOSITORY", repository_env);
    }
    
    let repository_env = std::env::var("GITHUB_REPOSITORY")
        .expect("GITHUB_REPOSITORY env variable is required to create a release on GitHub. This should be a Default Variable created by github.com.");

    let octocrab: octocrab::Octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()
        .expect("Failed to create Octocrab instance.");

    let (owner, repo) = repository_env.split_once("/")
        .expect("Failed to split the repository into owner and repo.");

    
    let version = release.version.to_string();
    
    info!("Creating Release: {}", version);
    
    if !args.dry_run
    {
        let tag = repository.find_tag(tag_oid.clone()).expect("Failed to find the tag.");
        let commit = repository.find_commit(release.commit).expect("Failed to find the commit.");
        
        let result = octocrab
            .repos(owner, repo)
            .releases()
            .create(version.as_str())
            .name(version.as_str())
            .body(tag.message().unwrap())
            .draft(false)
            .prerelease(release.tag == ReleaseType::PreRelease)
            .target_commitish(commit.id().to_string().as_str())
            .send().await;
    
        if let Err(error) = result
        {
            error!("Failed to create release: {:?}", error);
            return Err("Failed to create release.");
        }
    }

    Ok(())
}