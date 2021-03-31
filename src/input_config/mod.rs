use std::collections::HashMap;

use uuid::Uuid;

use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, Button, ButtonExt, ContainerExt, Label, ScrolledWindow, StateFlags, WidgetExt,
};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;
use wvr_data::config::project_config::{InputConfig, Speed};

pub mod cam_view;
pub mod midi_view;
pub mod picture_view;
pub mod video_view;

pub fn build_list_view(
    relm: &Relm<crate::Win>,
    input_config_widget_list: &mut HashMap<
        Uuid,
        (String, InputConfig, Component<InputConfigView>, gtk::Box),
    >,
    input_config_list: &HashMap<String, InputConfig>,
) -> (gtk::Box, gtk::Box) {
    let input_list_panel = gtk::Box::new(Vertical, 2);
    input_list_panel.set_property_margin(4);

    let input_list_control_container = gtk::Box::new(Horizontal, 8);
    input_list_control_container.set_property_margin(8);

    let add_cam_button = Button::new();
    add_cam_button.set_label("Add Camera");
    add_cam_button.set_hexpand(true);
    connect!(
        relm,
        add_cam_button,
        connect_clicked(_),
        Some(crate::Msg::AddCamInput)
    );

    let add_video_button = Button::new();
    add_video_button.set_label("Add Video");
    add_video_button.set_hexpand(true);
    connect!(
        relm,
        add_video_button,
        connect_clicked(_),
        Some(crate::Msg::AddVideoInput)
    );

    let add_picture_button = Button::new();
    add_picture_button.set_label("Add Picture");
    add_picture_button.set_hexpand(true);
    connect!(
        relm,
        add_picture_button,
        connect_clicked(_),
        Some(crate::Msg::AddPictureInput)
    );

    let add_midi_button = Button::new();
    add_midi_button.set_label("Add Midi Input");
    add_midi_button.set_hexpand(true);
    connect!(
        relm,
        add_midi_button,
        connect_clicked(_),
        Some(crate::Msg::AddMidiInput)
    );

    input_list_control_container.add(&add_cam_button);
    input_list_control_container.add(&add_video_button);
    input_list_control_container.add(&add_picture_button);
    input_list_control_container.add(&add_midi_button);

    let input_list_container = gtk::Box::new(Vertical, 16);
    input_list_container.set_property_margin(8);
    //input_list_container.override_background_color(StateFlags::NORMAL, Some(&RGBA::black()));

    for (input_name, input_config) in input_config_list.iter() {
        let (id, wrapper, input_config_view) =
            build_input_config_row(relm, input_name, &input_config);
        input_list_container.add(&wrapper);
        input_config_widget_list.insert(
            id,
            (
                input_name.clone(),
                input_config.clone(),
                input_config_view,
                wrapper,
            ),
        );
    }

    let input_list_container_wrapper = ScrolledWindow::new(
        Some(&Adjustment::new(320.0, 320.0, 10000.0, 1.0, 1.0, 1.0)),
        Some(&Adjustment::new(320.0, 320.0, 100000.0, 0.0, 0.0, 1.0)),
    );
    input_list_container_wrapper.set_size_request(480, 320);
    input_list_container_wrapper.set_hexpand(true);
    input_list_container_wrapper.set_vexpand(true);
    input_list_container_wrapper.add(&input_list_container);

    input_list_panel.add(&input_list_container_wrapper);
    input_list_panel.add(&input_list_control_container);

    (input_list_panel, input_list_container)
}

pub fn build_input_config_row(
    relm: &Relm<crate::Win>,
    input_name: &str,
    input_config: &InputConfig,
) -> (Uuid, gtk::Box, Component<InputConfigView>) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 2);
    let (label_name, label_color) = match input_config {
        InputConfig::Cam { .. } => (
            "Camera",
            RGBA {
                red: 0.0,
                green: 0.0,
                blue: 1.0,
                alpha: 0.125,
            },
        ),
        InputConfig::Video { .. } => (
            "Video",
            RGBA {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.125,
            },
        ),
        InputConfig::Picture { .. } => (
            "Picture",
            RGBA {
                red: 0.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.125,
            },
        ),
        InputConfig::Midi { .. } => (
            "Midi",
            RGBA {
                red: 1.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.125,
            },
        ),
    };

    let row_label = Label::new(Some(label_name));
    row_label.set_size_request(64, 0);
    row_label.override_background_color(StateFlags::NORMAL, Some(&label_color));

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    connect!(
        relm,
        remove_button,
        connect_clicked(_),
        Some(crate::Msg::RemoveInput(id))
    );

    wrapper.add(&row_label);
    let input_config_view = wrapper.add_widget::<InputConfigView>((
        id,
        input_name.to_string(),
        input_config.clone(),
        relm.clone(),
    ));
    wrapper.add(&remove_button);

    (id, wrapper, input_config_view)
}

#[derive(Msg)]
pub enum InputConfigViewMsg {
    SetName(String),
    SetWidth(i64),
    SetHeight(i64),
    SetPath(String),
    SetSpeedIsBpm(bool),
    SetSpeed(f64),
}

pub struct InputConfigViewModel {
    parent_relm: Relm<crate::Win>,
    id: Uuid,
    name: String,
    config: InputConfig,
}
pub struct InputConfigView {
    model: InputConfigViewModel,
    root: gtk::Box,
}

impl Update for InputConfigView {
    type Model = InputConfigViewModel;
    type ModelParam = (Uuid, String, InputConfig, Relm<crate::Win>);
    type Msg = InputConfigViewMsg;

    fn model(_: &Relm<Self>, model: (Uuid, String, InputConfig, Relm<crate::Win>)) -> Self::Model {
        InputConfigViewModel {
            id: model.0,
            name: model.1,
            config: model.2,
            parent_relm: model.3,
        }
    }

    fn update(&mut self, event: InputConfigViewMsg) {
        match &mut self.model.config {
            InputConfig::Cam {
                path,
                width,
                height,
            } => match event {
                InputConfigViewMsg::SetName(new_name) => self.model.name = new_name,
                InputConfigViewMsg::SetPath(new_path) => *path = new_path,
                InputConfigViewMsg::SetHeight(new_height) => *height = new_height as usize,
                InputConfigViewMsg::SetWidth(new_width) => *width = new_width as usize,
                _ => unreachable!(),
            },
            InputConfig::Video {
                path,
                width,
                height,
                speed,
            } => match event {
                InputConfigViewMsg::SetName(new_name) => self.model.name = new_name,
                InputConfigViewMsg::SetPath(new_path) => *path = new_path,
                InputConfigViewMsg::SetHeight(new_height) => *height = new_height as usize,
                InputConfigViewMsg::SetWidth(new_width) => *width = new_width as usize,
                InputConfigViewMsg::SetSpeed(new_speed) => {
                    *speed = match speed {
                        Speed::Beats(_) => Speed::Beats(new_speed as f32),
                        Speed::Fps(_) => Speed::Fps(new_speed as f32),
                    }
                }

                InputConfigViewMsg::SetSpeedIsBpm(speed_is_bpm) => {
                    let old_speed = match speed {
                        Speed::Beats(speed) => *speed,
                        Speed::Fps(speed) => *speed,
                    };
                    *speed = if speed_is_bpm {
                        Speed::Beats(old_speed)
                    } else {
                        Speed::Fps(old_speed)
                    };
                }
            },

            InputConfig::Picture {
                path,
                width,
                height,
            } => match event {
                InputConfigViewMsg::SetName(new_name) => self.model.name = new_name,
                InputConfigViewMsg::SetPath(new_path) => *path = new_path,
                InputConfigViewMsg::SetHeight(new_height) => *height = new_height as usize,
                InputConfigViewMsg::SetWidth(new_width) => *width = new_width as usize,
                _ => unreachable!(),
            },
            InputConfig::Midi { name } => match event {
                InputConfigViewMsg::SetName(new_name) => self.model.name = new_name,
                InputConfigViewMsg::SetPath(new_path) => *name = new_path,
                _ => unreachable!(),
            },
        }

        self.model
            .parent_relm
            .stream()
            .emit(crate::Msg::UpdateInputConfig(
                self.model.id,
                self.model.name.clone(),
                self.model.config.clone(),
            ));
    }
}

impl Widget for InputConfigView {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = match model.config {
            InputConfig::Cam { .. } => cam_view::build_cam_view(relm, &model),
            InputConfig::Video { .. } => video_view::build_video_view(relm, &model),
            InputConfig::Midi { .. } => midi_view::build_midi_view(relm, &model),
            InputConfig::Picture { .. } => picture_view::build_picture_view(relm, &model),
        };

        Self { model, root }
    }
}