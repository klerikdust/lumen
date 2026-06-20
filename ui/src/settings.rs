use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub struct UserSettings {
    pub always_on_top: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            always_on_top: true,
        }
    }
}

impl UserSettings {
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        let mut settings = Self::default();

        for line in contents.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            if key.trim() == "always_on_top" {
                settings.always_on_top = parse_bool(value.trim()).unwrap_or(settings.always_on_top);
            }
        }

        settings
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(path) = settings_path() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, format!("always_on_top={}\n", self.always_on_top))
    }
}

fn settings_path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .or_else(|| env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
        .map(|base| base.join(Path::new("Lumen").join("settings.conf")))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
