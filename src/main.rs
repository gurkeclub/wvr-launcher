//#![windows_subsystem = "windows"]

use anyhow::Result;

use relm::Widget;

mod config_panel;
mod input_config;
mod main_window;
mod server_config;
mod stage_config;
mod utils;
mod view_config;
mod welcome_panel;
mod wvr_frame;

use main_window::MainWindow;

pub fn main() -> Result<()> {
    MainWindow::run(()).expect("Failed to create main window");

    Ok(())
}
