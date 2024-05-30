use std::fmt::Display;

use log::debug;


#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub enum CommitType
{
    Major,
    Minor,
    Patch,
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct SemanticVersion
{
    major: u32,
    minor: u32,
    patch: u32,

    delta_major: u32,
    delta_minor: u32,
    delta_patch: u32,

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
        SemanticVersion { major: 0, minor: 0, patch: 0, delta_major: 0, delta_minor: 0, delta_patch: 0, prefix: None, suffix: None }
    }

    pub fn from(major: u32, minor: u32, patch: u32) -> SemanticVersion
    {
        SemanticVersion { major, minor, patch, delta_major: 0, delta_minor: 0, delta_patch: 0, prefix: None, suffix: None }
    }

    // getters
    pub fn get_major(&self) -> u32 { self.major }
    pub fn get_minor(&self) -> u32 { self.minor }
    pub fn get_patch(&self) -> u32 { self.patch }

    // Increment
    pub fn increment(&mut self, commit_type: &CommitType)
    {
        self.increment_by(commit_type, 1)
    }
    pub fn increment_by(&mut self, commit_type: &CommitType, value: u32)
    {
        match commit_type
        {
            CommitType::Major => { self.major += value; self.delta_major; self.minor = 0; self.delta_minor = 0; self.patch = 0; },
            CommitType::Minor => { self.minor += value; self.delta_major += value; self.patch = 0; self.delta_patch = 0; },
            CommitType::Patch => { self.patch += value; self.delta_patch += value; },
        }
    }

    pub fn get_delta_major(&self) -> u32 { self.delta_major }
    pub fn get_delta_minor(&self) -> u32 { self.delta_minor }
    pub fn get_delta_patch(&self) -> u32 { self.delta_patch }

    // Parse
    pub fn parse(version: &str) -> SemanticVersion
    {
        debug!("Parsing version: {}", version);

        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;

        let parts = version.split('-').collect::<Vec<&str>>();
        let version_part_index = if parts.len() > 1 { 1 } else { 0 };

        let version_parts = parts[version_part_index].split('.').collect::<Vec<&str>>();
        if !version_parts.is_empty()
        {
            // Remove any characters on major part and only leave digits.
            let major_part = version_parts[0].chars().filter(|c| c.is_digit(10)).collect::<String>();
            debug!("Major: {} [{}]", major_part, version_parts[0]);
            major = major_part.parse::<u32>().unwrap();
        }
        if version_parts.len() > 1
        {
            // Remove any characters on minor part.
            let minor_part = version_parts[1].chars().filter(|c| c.is_digit(10)).collect::<String>();
            debug!("Minor: {} [{}]", minor_part, version_parts[1]);
            minor = minor_part.parse::<u32>().unwrap();
        }
        if version_parts.len() > 2
        {
            // Remove any characters on patch part.
            let patch_part = version_parts[2].chars().filter(|c| c.is_digit(10)).collect::<String>();
            debug!("Patch: {} [{}]", patch_part, version_parts[2]);
            patch = patch_part.parse::<u32>().unwrap();
        }

        let prefix = if parts.len() > 1 { Some(parts[0].to_string()) } else { None };
        let suffix = if parts.len() > 2 { Some(parts[2].to_string()) } else { None };
        
        SemanticVersion { major, minor, patch, delta_major: 0, delta_minor: 0, delta_patch: 0, prefix, suffix }
    }
}

impl Display for SemanticVersion
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut version = self.major.to_string();
        
        // x[.x] or x[.x[.x]]; Optional minor and patch parts.
        if self.minor != u32::MAX
        {
            version = format!("{}.{}", version, self.minor);
        }
        if self.patch != u32::MAX
        {
            version = format!("{}.{}", version, self.patch);
        }

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