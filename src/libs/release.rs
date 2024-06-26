use git2::Oid;

use super::version::SemanticVersion;


#[derive(Debug, Clone)]
pub struct ReleaseContributor
{
    pub name: String,
    pub email: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseType
{
    None,
    Release,
    PreRelease,
}

#[derive(Debug, Clone)]
pub struct Release
{
    pub commit:         Oid,  
    pub tag:            ReleaseType,
    pub version:        SemanticVersion,
    pub majors:         Vec<String>,
    pub minors:         Vec<String>,
    pub patches:        Vec<String>,
    pub contributors:   Vec<ReleaseContributor>,
}
