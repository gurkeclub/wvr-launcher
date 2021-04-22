use nfd2::Response;

use wvr_data::config::project_config::{InputConfig, Speed};

use crate::config_panel::msg::ConfigPanelMsg;

pub fn create_video_input_config() -> Option<ConfigPanelMsg> {
    loop {
        match nfd2::open_file_dialog(None, None).expect("oh no") {
            Response::Okay(selected_file) => {
                if selected_file.exists() {
                    return Some(ConfigPanelMsg::AddInput(
                        "New Video".to_string(),
                        InputConfig::Video {
                            path: selected_file.to_str().unwrap().to_string(),
                            width: 640,
                            height: 480,
                            speed: Speed::Fps(25.0),
                        },
                    ));
                }
            }
            Response::OkayMultiple(_) => (),
            Response::Cancel => return None,
        }
    }
}

pub fn create_picture_input_config() -> Option<ConfigPanelMsg> {
    loop {
        match nfd2::open_file_dialog(None, None).expect("oh no") {
            Response::Okay(selected_file) => {
                if selected_file.exists() {
                    return Some(ConfigPanelMsg::AddInput(
                        "New Picture".to_string(),
                        InputConfig::Picture {
                            path: selected_file.to_str().unwrap().to_string(),
                            width: 640,
                            height: 480,
                        },
                    ));
                }
            }
            Response::OkayMultiple(_) => (),
            Response::Cancel => return None,
        }
    }
}
