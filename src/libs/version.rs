use std::fmt::Display;


#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub enum CommitType
{
    Major,
    Minor,
    Patch,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SemanticVersion
{
    major: u32,
    minor: u32,
    patch: u32,

    // Prefix & Suffix
    prefix: Option<String>,
    suffix: Option<String>,
}

impl PartialEq for SemanticVersion
{
    fn eq(&self, other: &Self) -> bool
    {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl SemanticVersion
{
    // Ctor
    pub fn new() -> SemanticVersion
    {
        SemanticVersion { major: 0, minor: 0, patch: 0, prefix: None, suffix: None }
    }

    // Increment
    pub fn increment(&mut self, commit_type: &CommitType)
    {
        self.increment_by(commit_type, 1)
    }
    pub fn increment_by(&mut self, commit_type: &CommitType, value: u32)
    {
        match commit_type
        {
            CommitType::Major => { self.major += value; self.minor = 0; self.patch = 0; },
            CommitType::Minor => { self.minor += value; self.patch = 0; },
            CommitType::Patch => self.patch += value,
        }
    }

    // Parse
    pub fn parse(version: &str) -> SemanticVersion
    {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;

        let parts = version.split('-').collect::<Vec<&str>>();
        let version_part_index = if parts.len() > 1 { 1 } else { 0 };

        let version_parts = parts[version_part_index].split('.').collect::<Vec<&str>>();
        if !version_parts.is_empty()
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

impl Display for SemanticVersion
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        write!(f, "{}", version)
    }
}