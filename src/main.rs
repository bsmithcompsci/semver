#![cfg_attr(feature = "strict", deny(missing_docs))]
#![cfg_attr(feature = "strict", deny(warnings))]
#![allow(dead_code)]
//! # Semantic Versioning
//! 
//! This is a simple tool to help automate the process of creating semantic versioning tags and releases for your repository.
//! 
//! ## Features
//! 
//! - [x] Semantic Versioning
//! - [x] Tagging
//! - [ ] Generation of Changelog
//! - [x] Release
//! - [x] Github
//! - [ ] Gitlab
//! - [ ] Bitbucket
//! - [ ] Gitea
//! 
//! ## Usage
//! 
//! ```bash
//! # Help
//! semver --help
//! 
//! # Version
//! semver --version
//! 
//! # Release the latest commit regardless, if the commit is tagged for release or not.
//! semver --input-file .semver.json --repository . --release
//! 
//! # Release the latest commit as a pre-release regardless, if the commit is tagged for pre-release or not.
//! semver --input-file .semver.json --repository . --prerelease
//! 
//! # Dry Run, do not act on anything, but give the outcome if it would.
//! semver --input-file .semver.json --repository . --dry-run
//! 
//! # Increment regardless, if there will be a release or not. This will skip versions in tags.
//! semver --input-file .semver.json --repository . --always-increment
//! 
//! # Skip any commits that are not formatted under the https://semver.org/ format rules. This will not include the skipped commits in the release.
//! semver --input-file .semver.json --repository . --skip-non-formatted
//! 
//! # Exit with an Error Code when encountering any errors; this is useful for CI/CD pipelines. This also implies non-formatted commits would cause errors.
//! semver --input-file .semver.json --repository . --exit-on-error
//! 
//! # Path to the credentials file. Default will go to your {HOME}/.ssh/Github
//! semver --input-file .semver.json --repository . --credentials ~/.ssh/Github
//! 
//! # Override the repository type: github, gitlab, bitbucket, gitea, etc.
//! semver --input-file .semver.json --repository . --override-repository-type github
//! 
//! # Override the repository type: github, gitlab, bitbucket, gitea, etc.
//! semver --input-file .semver.json --repository . --override-repository-type gitlab
//! 
//! # Override the repository type: github, gitlab, bitbucket, gitea, etc.
//! semver --input-file .semver.json --repository . --override-repository-type bitbucket
//! 
//! # Override the repository type: github, gitlab, bitbucket, gitea, etc.
//! semver --input-file .semver.json --repository . --override-repository-type gitea
//! ```
//! 
//! ## Configuration
//! 
//! ```json
//! {
//!    "tagging": {
//!       "supported_repositories": {
//!         "github": {
//!          "enabled": true
//!        }
//!     }
//!  }
//! }
//! ```
//! 
//! ## License
//! 
//! MIT
use std::{fs::File, io};

use clap::Parser;
use log::{debug, error, info};

mod libs;
mod feature;

use libs::data::*;
use maplit::hashmap;

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Path to the configuration file. Supports: .json", default_value = ".semver.json")]
    input_file: Option<String>,

    #[arg(short, long, help = "Directory of the targeted repository.", default_value = ".")]
    repository: Option<String>,

    #[arg(long, help = "Override the repository type: github, gitlab, bitbucket, gitea, etc.")]
    override_repository_type: Option<String>,

    #[arg(long, action, help = "Force the latest commit to be a release.", default_value = "false")]
    force_release: bool,
    #[arg(long, action, help = "Force the latest commit to be a pre-release", default_value = "false")]
    force_prerelease: bool,

    #[arg(long, action, help = "Do not act on anything, but give the outcome if it would.", default_value = "false")]
    dry_run: bool,

    #[arg(long, action, help = "Increments regardless, if there will be a release or not. This will skip versions in tags.", default_value = "false")]
    always_increment: bool,

    #[arg(long, action, help = "Skip any commits that are not formatted under the https://semver.org/ format rules.", default_value = "false")]
    skip_non_formatted: bool,

    #[arg(long, action, help = "Keeps the major version up to date with the latest of this set version.", default_value = "false")]
    keep_major_up_to_date: bool,

    #[arg(long, action, help = "Keeps the minor version up to date with the latest of this set version.", default_value = "false")]
    keep_minor_up_to_date: bool,

    #[arg(long, action, help = "Exit with an Error Code when encountering any errors.", default_value = "true")]
    exit_on_error: bool,

    #[arg(short, long, help = "Path to the credentials file. Default will go to your {HOME}/.ssh/Github")]
    credentials: Option<String>,
}

impl Clone for Args
{
    fn clone(&self) -> Self 
    {
        Args 
        {
            input_file: self.input_file.clone(),
            repository: self.repository.clone(),
            override_repository_type: self.override_repository_type.clone(),
            force_release: self.force_release,
            force_prerelease: self.force_prerelease,
            dry_run: self.dry_run,
            always_increment: self.always_increment,
            skip_non_formatted: self.skip_non_formatted,
            keep_major_up_to_date: self.keep_major_up_to_date,
            keep_minor_up_to_date: self.keep_minor_up_to_date,
            exit_on_error: self.exit_on_error,
            credentials: self.credentials.clone(),
        }
    }
}


#[tokio::main]
async fn main() {
    // Initialize the logger, while in debug mode, log everything; otherwise, log only errors, warnings and info.
    if cfg!(debug_assertions) 
    {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } 
    else 
    {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    // Parse the command line arguments
    let mut args = Args::parse();

    // Check if the JSON file path is provided
    let json_file: String = if let Some(json_input) = args.input_file.clone() {
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
        std::process::exit(1);
    }

    // Parse the JSON file with Serde
    let file = File::open(&json_file).unwrap();
    let reader = io::BufReader::new(file);
    let data: serde_json::Value = serde_json::from_reader(reader).unwrap();

    // Parse the JSON data into SemverData
    let semver_data: SemverData = serde_json::from_value(data).unwrap();
    info!("Read Semantic Version Data");
    // Check if the repository is provided
    let repository_base_path: String = if let Some(repository_base_path) = args.repository.clone() 
    {
        if repository_base_path.is_empty() 
        {
            error!("Repository is empty!");
            std::process::exit(1);
        }

        repository_base_path.clone()
    } 
    else 
    {
        String::from(".")
    };

    // Credentials
    if args.credentials.is_none()
    {
        let home = std::env::var("HOME").unwrap_or(std::env::var("USERPROFILE").unwrap_or(".".to_string()));
        args.credentials = Some(format!("{}/.ssh/Github", home));
    }
    
    if args.credentials.is_some()
    {
        std::env::set_var("GIT_SSH_KEY_PATH", args.credentials.clone().unwrap());

        debug!("Git Credentials Authenticated: {}", args.credentials.clone().unwrap());
    }

    let repository = git2::Repository::open(repository_base_path).unwrap();
    if repository.is_bare()
    {
        error!("Repository is bare!");
        std::process::exit(1);
    }

    let releases = feature::retrieval::get(
        args.clone(), 
        &semver_data, 
        &repository
    );

    info!("Releases: {}", releases.len());

    let repository_types = hashmap! {
        "github.com" => "github",
        // "gitlab.com" => "gitlab",
        // "bitbucket.org" => "bitbucket"
    };
    
    // Get Remote Origin URL
    let remote = repository.find_remote("origin").unwrap();
    let remote_url = remote.url().unwrap();

    let mut repository_type: Option<String> = None; 
    for (key, value) in repository_types.iter()
    {
        if remote_url.contains(key)
        {
            repository_type = Some(value.to_string());
            break;
        }
    };

    if repository_type.is_none()
    {
        error!("Repository Type is not supported: {}", remote_url);
        std::process::exit(1);
    }

    debug!("Repository Type: {} - {}", repository_type.clone().unwrap(), remote_url);

    // Tag the commits
    for release in releases.iter()
    {
        let commit = repository.find_commit(release.commit).unwrap();

        // Tag the release commits.
        if let Some(tag) = feature::tagging::tag(args.clone(), release, &commit, &repository)
        {
            // Publish a release to the appropriate repository.
            if semver_data.tagging.supported_repositories.contains_key(repository_type.clone().unwrap().as_str())
            {
                let repository_data = semver_data.tagging.supported_repositories.get(repository_type.clone().unwrap().as_str()).unwrap();
                if repository_data.enabled
                {
                    if let Err(error) = feature::release::create(args.clone(), repository_type.clone().unwrap().as_str(), release, &tag, &repository).await
                    {
                        error!("Failed to create release: {:?}", error);
                    
                        if args.exit_on_error
                        {
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

    }
}


/// Git Credentials Callback
/// 
/// This function is used to handle the Git Credentials Callback for the Git2 library.
/// 
/// ```rust
/// fn main() {
///     let user = "git";
///     let user_from_url = Some("git");
///     let cred = git2::CredentialType::USERNAME;
/// 
///     let result = crate::git_credentials_callback(user, user_from_url, cred);
/// 
///     match result {
///         Ok(cred) => {
///             println!("Cred: {:?}", cred);
///         },
///         Err(error) => {
///             println!("Error: {:?}", error);
///         }
///     }
///}
/// ```
pub(crate) fn git_credentials_callback(
    _user: &str,
    _user_from_url: Option<&str>,
    _cred: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let user = _user_from_url.unwrap_or("git");
    
    let _cred = match std::env::var("GIT_CREDENTIALS") {
        Ok(cred) => match cred.as_str() {
            "USERNAME"              => git2::CredentialType::USERNAME,
            "USER_PASS_PLAINTEXT"   => git2::CredentialType::USER_PASS_PLAINTEXT,
            "SSH_KEY"               => git2::CredentialType::SSH_KEY,
            _ => _cred,
        }
        _ => _cred,
    };

    debug!(r#"Git Credentials Callback:
        User: {}
        User From URL: {:?}
        Credential Type: {:?}
    "#, user, _user_from_url, _cred);


    // Handle the username.
    if _cred.contains(git2::CredentialType::USERNAME) {
        debug!("Authenticate with user {}", user);
        return git2::Cred::username(user);
    }

    // Handle the user and password.
    if _cred.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
        debug!("Authenticate with user {} and password", user);
        // Check if the user and token alias exists to set the git credential environments.
        //  We handle alias environments for the use to redirect the user and token to the corrent environment variables, 
        //  like GITHUB_USER and GITHUB_TOKEN; which are provided by Github. Since we are not specific for Github, we must use an alias.
        if let Ok(user) = std::env::var("GIT_ALIAS_USER") {
            std::env::set_var("GIT_USER", std::env::var(user.clone()).unwrap_or_else(|_| panic!("Missing Git Alias User output value: {}", user.clone())));
            debug!("GIT_ALIAS_USER was Set: {}", user);
        }
        if let Ok(token) = std::env::var("GIT_ALIAS_TOKEN") {
            std::env::set_var("GIT_TOKEN", std::env::var(token.clone()).unwrap_or_else(|_| panic!("Missing Git Alias Token output value: {}", token.clone())));
            debug!("GIT_ALIAS_TOKEN was Set: {}", token);
        }

        let username = std::env::var("GIT_USER").expect("Missing Git User");
        let password = std::env::var("GIT_TOKEN").expect("Missing Git Token");

        debug!("Username: {}", username);
        debug!("Password: {}", password);

        // Login with the user and token.
        return git2::Cred::userpass_plaintext(
            username.as_str(),
            password.as_str()
        );
    }

    // Handle the user and private key either via path or in-memory.
    match std::env::var("GIT_SSH_KEY_PATH") {
        Ok(private_key_path) => {
            debug!("Authenticate with user {} and private key located in {}", user, private_key_path);

            // Check if the public key exists.
            let public_key = private_key_path.clone() + ".pub";
            let public_key_path = if !std::path::Path::new(&public_key).exists() 
            {
                Some(std::path::Path::new(&public_key))
            }
            else
            {
                None
            };

            // Check if the private key exists.
            if !std::path::Path::new(&private_key_path).exists() 
            {
                return Err(git2::Error::from_str(format!("GIT_SSH_KEY path does not exist: {}", private_key_path).as_str()));
            }
            
            git2::Cred::ssh_key(user, public_key_path, std::path::Path::new(&private_key_path), None)
        },
        _ => match std::env::var("GIT_SSH_KEY") {
            Ok(private_key) => {
                debug!("Authenticate with user {} and private key in memory", user);
    
                // Check if the public key exists.
                let public_key = std::env::var("GIT_SSH_KEY_PUBLIC");
                let public_key = match public_key {
                    Ok(public_key) => Some(public_key),
                    _ => None,
                };
                let public_key = public_key.as_deref();
                
                git2::Cred::ssh_key_from_memory(user, public_key, &private_key, None)
            },
            _ => Err(git2::Error::from_str("unable to get private key from GIT_SSH_KEY")),
        },
    }
    
}