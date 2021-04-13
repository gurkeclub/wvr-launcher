//#![windows_subsystem = "windows"]

use anyhow::Result;
use uuid::Uuid;

use relm::Widget;
use relm_derive::Msg;

use wvr_data::config::project_config::{InputConfig, RenderStageConfig};

mod input_config;
mod main_panel;
mod main_window;
mod server_config;
mod stage_config;
mod view_config;

use main_window::MainWindow;

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

pub fn main() -> Result<()> {
    MainWindow::run(()).expect("Failed to create main window");

    Ok(())
}
