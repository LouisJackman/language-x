/// Versioning across Sylan is done consistently with [Semantic Versioning](https://semver.org), aka
/// "semver".
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}
