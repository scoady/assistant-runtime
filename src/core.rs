use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfile {
    Auto,
    Desktop,
    Server,
}

impl RuntimeProfile {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "desktop" => Ok(Self::Desktop),
            "server" => Ok(Self::Server),
            other => Err(format!("unknown profile: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeImage {
    Desktop,
    Server,
}

impl RuntimeImage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Server => "server",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnvironment {
    pub os: String,
    pub arch: String,
    pub has_gui_display: bool,
}

impl RuntimeEnvironment {
    pub fn detect() -> Self {
        let has_gui_display = env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some() || cfg!(target_os = "macos");
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            has_gui_display,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBootPlan {
    pub profile: RuntimeProfile,
    pub image: RuntimeImage,
    pub reason: String,
    pub port: u16,
}

pub fn build_boot_plan(env: &RuntimeEnvironment, profile: RuntimeProfile, port: u16) -> RuntimeBootPlan {
    match profile {
        RuntimeProfile::Desktop => RuntimeBootPlan { profile, image: RuntimeImage::Desktop, reason: "desktop profile explicitly requested".into(), port },
        RuntimeProfile::Server => RuntimeBootPlan { profile, image: RuntimeImage::Server, reason: "server profile explicitly requested".into(), port },
        RuntimeProfile::Auto => {
            if env.has_gui_display {
                RuntimeBootPlan { profile, image: RuntimeImage::Desktop, reason: "auto selected desktop image because a GUI-capable host is available".into(), port }
            } else {
                RuntimeBootPlan { profile, image: RuntimeImage::Server, reason: "auto selected server image because no GUI display is available".into(), port }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostReport {
    pub schema: &'static str,
    pub ok: bool,
    pub profile: RuntimeProfile,
    pub image: RuntimeImage,
    pub checks: Vec<PostCheck>,
}
