use crate::libs::release::Release;

pub mod github;

pub async fn create(args: crate::Args, repository_type: &str, release: &Release, tag_oid: &git2::Oid, repository: &git2::Repository) -> Result<(), &'static str>
{
    match repository_type
    {
        "github" => github::create(args, release, tag_oid, repository).await,
        _ => Err("Repository Type is not supported")
    }
}