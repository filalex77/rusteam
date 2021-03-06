use crate::filesystem::{entries, has_same_name_as_parent_dir};
use std::fmt;
use std::path::{Path, PathBuf};

/// A platform that a [`Game`] can be developed for.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Platform {
    Native,
    Wine,
}

/// A genre that a [`Game`] can belong to.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Genre {
    Action,
    Platformer,
}

/// A game on your hard drive.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Game {
    /// Name of the game. Can be inferred from the directory.
    pub name: Option<String>,
    /// Platform the game runs on: native (Linux), Wine, etc.
    pub platform: Option<Platform>,
    /// Where the game is located:
    /// the root directory that contains game files and launchers.
    pub directory: PathBuf,
    /// Currently unused, but may be useful for filtering.
    pub genres: Vec<Genre>,
    /// Paths to executable files that start the game.
    pub launchers: Vec<PathBuf>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let default = &String::from("a game with no name");
        let name = self.name.as_ref().unwrap_or(default);

        write!(f, "{}", name)
    }
}

impl Game {
    /// Constructs a [`Game`] from a [`PathBuf`].
    ///
    /// Most of the metadata about the game is inferred.
    /// Currently there is no way to customize the inferred data.
    pub fn from_path(directory: PathBuf) -> Self {
        let (platform, launchers) = Self::find_launchers(&directory);

        Self {
            // Name of the game is the name of its directory.
            name: Self::basename(&directory),
            // Genres is beyond us for now.
            genres: vec![],
            platform,
            launchers,
            directory,
        }
    }

    fn find_launchers(directory: &Path) -> (Option<Platform>, Vec<PathBuf>) {
        // We check for knows launchers in the root of the directory.

        let launchers = entries(directory)
            .into_iter()
            .filter(|filepath| Self::is_launcher(filepath))
            .collect::<Vec<PathBuf>>();

        // We can tell the platform if all found launchers belong to it.

        (Self::same_platform(launchers.as_slice()), launchers)
    }

    fn same_platform(launchers: &[PathBuf]) -> Option<Platform> {
        if launchers.is_empty() {
            None
        } else {
            Self::platform(&launchers[0]).filter(|first_platform| {
                launchers
                    .iter()
                    .all(|l| Self::platform(l).filter(|p| p == first_platform).is_some())
            })
        }
    }

    fn platform(file: &Path) -> Option<Platform> {
        match file {
            file if Self::is_native(file) => Some(Platform::Native),
            file if Self::is_wine(file) => Some(Platform::Wine),
            _ => None,
        }
    }

    fn is_launcher(filepath: &Path) -> bool {
        !Self::is_uninstall(filepath)
            && (Self::is_native(filepath)
                || Self::is_wine(filepath)
                || has_same_name_as_parent_dir(filepath))
    }

    /// Checks if file is an uninstaller.
    fn is_uninstall(file: &Path) -> bool {
        file.file_name()
            .map(|f| f.to_string_lossy())
            .map_or(false, |f| f.contains("uninstall"))
    }

    /// Checks if file is a native Linux executable (empirically).
    fn is_native(file: &Path) -> bool {
        Self::extension_in(file, &["sh", "x86", "x86_64"])
    }

    /// Checks if file is a Wine executable (empirically).
    fn is_wine(file: &Path) -> bool {
        Self::extension_in(file, &["exe"])
    }

    /// Checks if file has one of the extensions.
    fn extension_in(file: &Path, extensions: &[&str]) -> bool {
        file.extension()
            .map(|ext| ext.to_string_lossy())
            .map_or(false, |ext| extensions.contains(&ext.as_ref()))
    }

    /// Gets the basename out of a path.
    fn basename(path: &Path) -> Option<String> {
        path.file_name().and_then(|f| f.to_str()).map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uninstall() {
        assert!(Game::is_uninstall(Path::new("game/uninstall-game.sh")))
    }

    #[test]
    fn test_is_native() {
        assert!(Game::is_native(Path::new("game/run.sh")))
    }

    #[test]
    fn test_is_wine() {
        assert!(Game::is_wine(Path::new("win_game/launcher.exe")))
    }

    #[test]
    fn test_extension_in() {
        assert!(Game::extension_in(
            Path::new("/home/file.png"),
            &["jpg", "png"]
        ))
    }

    #[test]
    fn test_basename() {
        assert_eq!(
            Some("file.png".to_string()),
            Game::basename(Path::new("/home/file.png"))
        )
    }
}
