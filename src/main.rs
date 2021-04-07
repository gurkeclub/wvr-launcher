#![windows_subsystem = "windows"]

use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use uuid::Uuid;

use relm::Widget;
use relm_derive::Msg;

use nfd2::Response;

use wvr_data::config::project_config::ProjectConfig;
use wvr_data::config::project_config::{InputConfig, RenderStageConfig};

mod config_window;
mod input_config;
mod server_config;
mod stage_config;
mod view_config;

use config_window::Win;

#[derive(Msg, Debug)]
pub enum Msg {
    SetBpm(f64),
    SetWidth(i64),
    SetHeight(i64),
    SetTargetFps(f64),
    SetDynamicResolution(bool),
    SetVSync(bool),
    SetScreenshot(bool),
    SetFullscreen(bool),
    SetLockedSpeed(bool),

    SetServerIp(String),
    SetServerPort(i64),
    SetServerEnabled(bool),

    AddPictureInput,
    AddCamInput,
    AddVideoInput,
    AddMidiInput,
    UpdateInputConfig(Uuid, String, InputConfig),
    RemoveInput(Uuid),

    AddRenderStage,
    UpdateRenderStageConfig(Uuid, RenderStageConfig),
    RemoveRenderStage(Uuid),

    UpdateRenderedTextureSampling,
    UpdateRenderedTextureName,

    Quit,
    Save,
    Start,
    Error(String),
}

fn get_config() -> std::option::Option<(PathBuf, ProjectConfig)> {
    let wvr_data_path = wvr_data::get_data_path();

    let mut config_path = None;
    let projects_path = wvr_data_path.join("projects");

    while config_path.is_none() {
        match nfd2::open_file_dialog(None, Some(&projects_path)).expect("oh no") {
            Response::Okay(file_path) => config_path = Some(file_path),
            Response::OkayMultiple(_) => (),
            Response::Cancel => return None,
        }
    }

    let config_path = config_path.unwrap();

    let project_path = config_path.parent().unwrap().to_owned();
    let config: ProjectConfig = if let Ok(file) = File::open(&config_path) {
        serde_json::from_reader::<File, ProjectConfig>(file).unwrap()
    } else {
        panic!("Could not find config file {:?}", project_path);
    };

    Some((project_path, config))
}

pub fn main() -> Result<()> {
    if let Some(project) = get_config() {
        Win::run(project).expect("Win::run failed");
    }

    Ok(())
}
