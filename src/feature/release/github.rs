use crate::libs::release::{Release, ReleaseType};

use log::{debug, error, info};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DeleteReleaseParams
{
    owner: String,
    repo: String,
    release_id: u64,
}

pub async fn create(args: crate::Args, release: &Release, tag_oid: &git2::Oid, repository: &git2::Repository) -> Result<Option<octocrab::models::repos::Release>, &'static str>
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

        let (owner, repo) = repository_env.split_once('/').unwrap();
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

    let (owner, repo) = repository_env.split_once('/')
        .expect("Failed to split the repository into owner and repo.");

    
    let version = release.version.to_string();
    
    info!("Creating Release: {}", version);
    
    if args.dry_run
    {
        return Ok(None);
    }

    if tag_oid.is_zero()
    {
        return Err("Tag OID is Zero.");
    }

    let tag = repository.find_tag(*tag_oid).expect("Failed to find the tag.");
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

    Ok(Some(result.unwrap()))
}

#[tokio::test]
async fn test_create()
{
    // use crate::libs::version::SemanticVersion;
    // use crate::libs::release::Release;
    // use crate::libs::release::ReleaseType;
    // use crate::libs::release::ReleaseContributor;
    
    // let _ = env_logger::Builder::new()
    //     .filter_level(log::LevelFilter::Debug)
    //     .try_init();

    // {
    //     let home = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".to_string()));
    //     let var = Some(format!("{}/.ssh/Github", home));
    //     std::env::set_var("GIT_SSH_KEY_PATH", var.clone().unwrap());
    //     debug!("Git Credentials Authenticated: {}", var.clone().unwrap());
    // }

    // let rand_major = rand::random::<u8>();
    // let rand_minor = rand::random::<u8>();
    // let rand_patch = rand::random::<u8>();

    // let mut version = SemanticVersion::new();
    // version.increment_by(&CommitType::MAJOR, rand_major as u32);
    // version.increment_by(&CommitType::MINOR, rand_minor as u32);
    // version.increment_by(&CommitType::PATCH, rand_patch as u32);

    // let repository = git2::Repository::open(".").unwrap();
    // let commit = repository.head().unwrap().peel_to_commit().unwrap();
    // let release = Release {
    //     version: version.clone(),
    //     tag: ReleaseType::Release,
    //     commit: commit.id(),
    //     majors: vec!["Major Change".to_string()],
    //     minors: vec!["Minor Change".to_string()],
    //     patches: vec!["Patch Change".to_string()],
    //     contributors: vec![ReleaseContributor { name: "Name".to_string(), email: "Test@email.com".to_string() }],
    // };

    // let mut args = crate::Args::default();
    // args.dry_run = true;


    // let tag_oid = crate::feature::tagging::tag(args, &release, &commit, &repository);

    // assert!(tag_oid.is_some());
    // // assert!(tag_oid.unwrap().is_zero() == false);

    // let args = crate::Args::default();

    // let result = create(args, &release, &tag_oid.unwrap(), &repository).await;

    // assert!(result.is_ok());
    // assert!(result.clone().unwrap().is_some());

    // let tag_name = release.version.to_string();

    // // Cleanup when in test.
    // repository.tag_delete(&tag_name.as_str()).unwrap();
    
    // // Callbacks
    // let mut callbacks = git2::RemoteCallbacks::new();
    // callbacks.credentials(crate::git_credentials_callback);
    
    // // Push Tags.
    // let mut remote = repository.find_remote("origin").unwrap();
    // if let Err(error) = remote.push(&[format!("refs/tags/{}", tag_name.as_str()).to_string()], Some(git2::PushOptions::new().remote_callbacks(callbacks)))
    // {
    //     error!("Failed to push Tag: {} for {}\n\t{:?}", tag_name.as_str(), commit.id(), error);
    // }
    // else
    // {
    //     info!("Pushed Tag: {} for {}", tag_name.as_str(), commit.id());
    // }

    // {
    //     let repository_env = repository.find_remote("origin")
    //             .expect("Failed to find the remote origin.")
    //             .url()
    //             .expect("Failed to get the remote origin URL.")
    //             .to_string();
    
    //         let (owner, repo) = repository_env.split_once("/").unwrap();
    //         let owner = owner.split_once("github.com:").unwrap().1;
    //         let repo = repo.replace(".git", "");
    
    //         let repository_env = format!("{}/{}", owner, repo);
    
    //         debug!("Loading Repository: {:?}", repository_env);
    
    //         std::env::set_var("GITHUB_REPOSITORY", repository_env);
    // }
    
    // let token = std::env::var("GITHUB_TOKEN")
    //     .expect("GITHUB_TOKEN env variable is required to create a release on GitHub. This should be a Default Variable created by github.com.");

    // let repository_env = std::env::var("GITHUB_REPOSITORY")
    //     .expect("GITHUB_REPOSITORY env variable is required to create a release on GitHub. This should be a Default Variable created by github.com.");

    // let octocrab: octocrab::Octocrab = octocrab::Octocrab::builder()
    //     .personal_token(token)
    //     .build()
    //     .expect("Failed to create Octocrab instance.");

    // let (owner, repo) = repository_env.split_once("/")
    //     .expect("Failed to split the repository into owner and repo.");


    // let delete_release: Result<octocrab::models::repos::Release, octocrab::Error> = octocrab
    //     .delete("repos/{owner}/{repo}/releases/{release_id}", Some(&DeleteReleaseParams { 
    //         owner: owner.to_string(), 
    //         repo: repo.to_string(), 
    //         release_id: result.unwrap().unwrap().id.0 
    //     }))
    //     .await;


    // // Wait until logs are completely sent.
    // std::thread::sleep(std::time::Duration::from_secs(1));

    // assert!(delete_release.is_ok());
}