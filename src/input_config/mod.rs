use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use gtk::{
    Adjustment, Button, ButtonExt, ContainerExt, Label, ScrolledWindow, ScrolledWindowExt,
    WidgetExt,
};
use gtk::{
    Orientation::{Horizontal, Vertical},
    PolicyType,
};

use relm::{connect, Relm};
use relm_derive::Msg;
use wvr_data::config::project_config::{InputConfig, Speed};

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;

pub mod cam_view;
pub mod midi_view;
pub mod picture_view;
mod utils;
pub mod video_view;

pub fn build_list_view(
    relm: &Relm<ConfigPanel>,
    project_path: &Path,
    input_config_widget_list: &mut HashMap<Uuid, (String, InputConfig, gtk::Box)>,
    input_config_list: &HashMap<String, InputConfig>,
) -> (gtk::Box, gtk::Box) {
    let input_list_panel = gtk::Box::new(Vertical, 4);
    input_list_panel.set_property_margin(8);

    let input_list_control_container = gtk::Box::new(Horizontal, 8);

    let add_cam_button = Button::new();
    add_cam_button.set_label("Add Camera");
    add_cam_button.set_hexpand(true);
    connect!(
        relm,
        add_cam_button,
        connect_clicked(_),
        Some(ConfigPanelMsg::AddInput(
            "New Camera".to_string(),
            InputConfig::Cam {
                path: "/dev/video0".to_string(),
                width: 640,
                height: 480,
            }
        ))
    );

    let add_video_button = Button::new();
    add_video_button.set_label("Add Video");
    add_video_button.set_hexpand(true);
    connect!(
        relm,
        add_video_button,
        connect_clicked(_),
        utils::create_video_input_config()
    );

    let add_picture_button = Button::new();
    add_picture_button.set_label("Add Picture");
    add_picture_button.set_hexpand(true);
    connect!(
        relm,
        add_picture_button,
        connect_clicked(_),
        utils::create_picture_input_config()
    );

    let add_midi_button = Button::new();
    add_midi_button.set_label("Add Midi Input");
    add_midi_button.set_hexpand(true);
    connect!(
        relm,
        add_midi_button,
        connect_clicked(_),
        Some(ConfigPanelMsg::AddInput(
            "New Controller".to_string(),
            InputConfig::Midi {
                name: "*".to_string(),
            }
        ))
    );

    input_list_control_container.add(&add_cam_button);
    input_list_control_container.add(&add_video_button);
    input_list_control_container.add(&add_picture_button);
    input_list_control_container.add(&add_midi_button);

    let input_list_container = gtk::Box::new(Vertical, 16);

    for (input_name, input_config) in input_config_list.iter() {
        let (id, wrapper) = build_input_config_row(relm, project_path, input_name, &input_config);

        input_list_container.add(&wrapper);
        input_config_widget_list.insert(id, (input_name.clone(), input_config.clone(), wrapper));
    }

    let input_list_container_wrapper = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    input_list_container_wrapper.set_policy(PolicyType::Never, PolicyType::Automatic);
    input_list_container_wrapper.set_hexpand(true);
    input_list_container_wrapper.set_vexpand(true);
    input_list_container_wrapper.add(&input_list_container);

    input_list_panel.add(&input_list_container_wrapper);
    input_list_panel.add(&input_list_control_container);

    (input_list_panel, input_list_container)
}

pub fn build_input_config_row(
    relm: &Relm<ConfigPanel>,
    project_path: &Path,
    input_name: &str,
    input_config: &InputConfig,
) -> (Uuid, gtk::Box) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 2);
    let label_name = match input_config {
        InputConfig::Cam { .. } => emoji::objects::light_and_video::VIDEO_CAMERA,
        InputConfig::Video { .. } => emoji::objects::light_and_video::FILM_FRAMES,
        InputConfig::Picture { .. } => emoji::activities::arts_and_crafts::FRAMED_PICTURE,
        InputConfig::Midi { .. } => emoji::objects::music::CONTROL_KNOBS,
    };

    let row_label = Label::new(Some(label_name));
    row_label.set_size_request(64, 0);

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    connect!(
        relm,
        remove_button,
        connect_clicked(_),
        Some(ConfigPanelMsg::RemoveInput(id))
    );

    wrapper.add(&row_label);
    let input_config_view = match input_config {
        InputConfig::Cam { .. } => cam_view::build_cam_view(relm, id, input_name, input_config),
        InputConfig::Video { .. } => {
            video_view::build_video_view(relm, &project_path, id, input_name, input_config)
        }
        InputConfig::Midi { .. } => midi_view::build_midi_view(relm, id, input_name, input_config),
        InputConfig::Picture { .. } => {
            picture_view::build_picture_view(relm, &project_path, id, input_name, input_config)
        }
    };

    wrapper.add(&input_config_view);
    wrapper.add(&remove_button);

    (id, wrapper)
}

#[derive(Msg, Debug)]
pub enum InputConfigViewMsg {
    SetName(String),
    SetWidth(i64),
    SetHeight(i64),
    SetPath(String),
    SetSpeed(Speed),
}
