//#![windows_subsystem = "windows"]

use anyhow::Result;

use relm::Widget;
use relm_derive::Msg;

mod input_config;
mod main_panel;
mod main_window;
mod server_config;
mod stage_config;
mod view_config;
mod welcome_panel;

use main_window::MainWindow;

pub fn main() -> Result<()> {
    MainWindow::run(()).expect("Failed to create main window");

    Ok(())
}
