use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct SemverDataTaggingRepository
{
    pub enabled: bool,
}
#[derive(serde::Deserialize, Debug)]
pub struct SemverDataTagging
{
    pub supported_repositories: HashMap<String, SemverDataTaggingRepository>,
}

#[derive(serde::Deserialize, Debug)]
pub struct SemverDataBranch
{
    pub name: String,
    pub prerelease: Option<bool>,
    pub increment: Option<Vec<String>>
}

#[derive(serde::Deserialize, Debug)]
pub struct SemverDataCommits
{
    pub default: String,
    #[serde(alias = "caseSensitive")]
    pub case_sensitive: bool,
    pub release: Vec<String>,
    pub prerelease: Vec<String>,
    pub map: HashMap<String, Vec<String>>
}

#[derive(serde::Deserialize, Debug)]
pub struct SemverData {
    pub tagging: SemverDataTagging,
    pub branches: Vec<SemverDataBranch>,
    pub commits: SemverDataCommits
}