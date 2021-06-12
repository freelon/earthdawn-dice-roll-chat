const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_COMMIT: &str = git_version::git_version!(fallback = "unknown");

pub fn welcome_message() -> String {
    format!("Welcome to the Earthdawn Dice Roll Chat.<br>Server version: {}. Build version: {}", PKG_VERSION, GIT_COMMIT)
}
