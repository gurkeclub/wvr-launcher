#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use uuid::Uuid;

use gtk::prelude::NotebookExtManual;
use gtk::prelude::WidgetExtManual;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Button, ButtonExt, ContainerExt, GtkWindowExt, Inhibit, Label, Notebook, WidgetExt, Window,
    WindowType,
};

use relm::{connect, Component, Relm, Update, Widget};
use relm_derive::Msg;

use nfd2::Response;

use wvr_data::config::project_config::ProjectConfig;
use wvr_data::config::project_config::{InputConfig, Speed};

mod input_config;
mod server_config;
mod view_config;

use input_config::InputConfigView;

#[derive(Msg, Debug)]
pub enum Msg {
    SetBPM(f64),
    SetWidth(i64),
    SetHeight(i64),
    SetTargetFps(f64),
    SetDynamicResolution(bool),
    SetVSync(bool),
    SetScreenshot(bool),
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

    Quit,
    Save,
    Start,
    Error(String),
}

pub struct Model {
    project_path: PathBuf,
    config: ProjectConfig,
}

pub struct Win {
    model: Model,

    window: Window,

    input_list_container: gtk::Box,
    input_config_widget_list:
        HashMap<Uuid, (String, InputConfig, Component<InputConfigView>, gtk::Box)>,

    relm: Relm<Self>,
}

impl Win {
    fn save_config(&mut self, project_config_file_path: &Path) {
        println!("{:?}", project_config_file_path);
        self.model.config.inputs.clear();
        for (name, config, _, _) in self.input_config_widget_list.values() {
            self.model
                .config
                .inputs
                .insert(name.clone(), config.clone());
        }

        let config_path = project_config_file_path;
        if let Ok(mut project_config_file) = std::fs::File::create(config_path) {
            let config_as_bytes = serde_json::ser::to_string_pretty(&self.model.config)
                .unwrap()
                .into_bytes();

            project_config_file.write_all(&config_as_bytes).unwrap();
        }
    }

    fn load_config(&mut self, project_config_file_path: &Path) {}

    fn start(&mut self) {
        let config_path = self.model.project_path.join("config.tmp.json");

        self.save_config(&config_path);

        self.window.hide();

        Command::new("wvr")
            .arg("-c")
            .arg(config_path.to_str().unwrap())
            .output()
            .expect("failed to execute process");

        self.window.show();
    }
}

impl Update for Win {
    type Model = Model;
    type ModelParam = (PathBuf, ProjectConfig);
    type Msg = Msg;

    fn model(_: &Relm<Self>, project: (PathBuf, ProjectConfig)) -> Self::Model {
        Model {
            project_path: project.0,
            config: project.1,
        }
    }

    fn update(&mut self, event: Msg) {
        println!("{:?}", event);
        match event {
            Msg::Quit => {
                unsafe {
                    self.window.destroy();
                }
                gtk::main_quit()
            }
            Msg::SetBPM(bpm) => self.model.config.bpm = bpm as f32,
            Msg::SetWidth(width) => self.model.config.view.width = width,
            Msg::SetHeight(height) => self.model.config.view.height = height,
            Msg::SetTargetFps(fps) => self.model.config.view.target_fps = fps as f32,
            Msg::SetDynamicResolution(dynamic) => self.model.config.view.dynamic = dynamic,
            Msg::SetVSync(vsync) => self.model.config.view.vsync = vsync,
            Msg::SetScreenshot(screenshot) => self.model.config.view.screenshot = screenshot,
            Msg::SetLockedSpeed(locked_speed) => self.model.config.view.locked_speed = locked_speed,

            Msg::SetServerIp(ip) => self.model.config.server.ip = ip,
            Msg::SetServerPort(port) => self.model.config.server.port = port as usize,
            Msg::SetServerEnabled(enable) => self.model.config.server.enable = enable,

            Msg::Save => {
                self.save_config(&self.model.project_path.join("config.json"));
            }
            Msg::Start => {
                self.start();
            }
            Msg::Error(e) => eprintln!("{:?}", e),

            Msg::AddCamInput => {
                let input_cam_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_cam())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Camera #{:}", input_cam_count + 1);

                let input_config = InputConfig::Cam {
                    path: "/dev/video0".to_string(),
                    width: 640,
                    height: 480,
                };

                let (id, wrapper, input_config_view) =
                    input_config::build_input_config_row(&self.relm, &input_name, &input_config);

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list.insert(
                    id,
                    (
                        input_name.to_string(),
                        input_config,
                        input_config_view,
                        wrapper,
                    ),
                );
            }
            Msg::AddVideoInput => {
                let input_video_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_video())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Video #{:}", input_video_count + 1);

                let input_config = InputConfig::Video {
                    path: "res/example_video.mp4".to_string(),
                    width: 640,
                    height: 480,
                    speed: Speed::Fps(25.0),
                };

                let (id, wrapper, input_config_view) =
                    input_config::build_input_config_row(&self.relm, &input_name, &input_config);

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list.insert(
                    id,
                    (
                        input_name.to_string(),
                        input_config,
                        input_config_view,
                        wrapper,
                    ),
                );
            }
            Msg::AddPictureInput => {
                let input_picture_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_picture())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Picture #{:}", input_picture_count + 1);

                let input_config = InputConfig::Picture {
                    path: "res/example_picture.png".to_string(),
                    width: 640,
                    height: 480,
                };

                let (id, wrapper, input_config_view) =
                    input_config::build_input_config_row(&self.relm, &input_name, &input_config);

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list.insert(
                    id,
                    (
                        input_name.to_string(),
                        input_config,
                        input_config_view,
                        wrapper,
                    ),
                );
            }
            Msg::AddMidiInput => {
                let input_midi_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_midi())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Midi #{:}", input_midi_count + 1);

                let input_config = InputConfig::Midi {
                    name: "*".to_string(),
                };

                let (id, wrapper, input_config_view) =
                    input_config::build_input_config_row(&self.relm, &input_name, &input_config);

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list.insert(
                    id,
                    (
                        input_name.to_string(),
                        input_config,
                        input_config_view,
                        wrapper,
                    ),
                );
            }
            Msg::RemoveInput(id) => {
                if let Some((_, _, _, input_view_wrapper)) = self.input_config_widget_list.get(&id)
                {
                    self.input_list_container.remove(input_view_wrapper);
                }
                self.input_config_widget_list.remove(&id);
            }
            Msg::UpdateInputConfig(id, new_name, new_config) => {
                if let Some((ref mut name, ref mut config, _, _)) =
                    self.input_config_widget_list.get_mut(&id)
                {
                    *name = new_name;
                    *config = new_config;
                }
            }
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let mut input_config_widget_list = HashMap::new();

        let model = model;
        let window = gtk::Window::new(WindowType::Toplevel);
        let window_container = gtk::Box::new(Vertical, 16);

        window.set_title("wvr launcher");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        let tabs_container = Notebook::new();

        let view_config_widget =
            view_config::build_view(relm, model.config.bpm as f64, &model.config.view);

        let server_config_panel = server_config::build_view(relm, &model.config.server);

        let (input_list_panel, input_list_container) = input_config::build_list_view(
            relm,
            &mut input_config_widget_list,
            &model.config.inputs,
        );

        let render_chain_panel = gtk::Box::new(Vertical, 0);
        // TODO

        let final_stage_panel = gtk::Box::new(Vertical, 0);
        // TODO

        tabs_container.append_page(&view_config_widget, Some(&Label::new(Some("View"))));
        tabs_container.append_page(&server_config_panel, Some(&Label::new(Some("Server"))));
        tabs_container.append_page(&input_list_panel, Some(&Label::new(Some("Inputs"))));
        tabs_container.append_page(&render_chain_panel, Some(&Label::new(Some("Render chain"))));
        tabs_container.append_page(&final_stage_panel, Some(&Label::new(Some("Final stage"))));

        let control_container = gtk::Box::new(Horizontal, 8);

        let save_button = Button::new();
        save_button.set_label("Save");
        save_button.set_hexpand(true);
        connect!(relm, save_button, connect_clicked(_), Some(Msg::Save));

        let start_button = Button::new();
        start_button.set_label("Start");
        start_button.set_hexpand(true);
        connect!(relm, start_button, connect_clicked(_), Some(Msg::Start));

        control_container.add(&save_button);
        control_container.add(&start_button);

        window_container.add(&tabs_container);
        window_container.add(&control_container);

        window.add(&window_container);

        window.show_all();

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        Win {
            model,

            window,

            input_list_container,

            input_config_widget_list,

            relm: relm.clone(),
        }
    }
}

fn get_config() -> Result<Option<(PathBuf, ProjectConfig)>> {
    let wvr_data_path = wvr_data::get_data_path();

    let mut config_path = None;
    let projects_path = wvr_data_path.join("projects");

    while config_path.is_none() {
        match nfd2::open_file_dialog(None, Some(&projects_path)).expect("oh no") {
            Response::Okay(file_path) => config_path = Some(file_path),
            Response::OkayMultiple(_) => (),
            Response::Cancel => return Ok(None),
        }
    }

    let config_path = config_path.unwrap();

    let project_path = config_path.parent().unwrap().to_owned();
    let config: ProjectConfig = if let Ok(file) = File::open(&config_path) {
        serde_json::from_reader::<File, ProjectConfig>(file).unwrap()
    } else {
        panic!("Could not find config file {:?}", project_path);
    };

    Ok(Some((project_path, config)))
}

pub fn main() -> Result<()> {
    if let Some(project) = get_config()? {
        Win::run(project).expect("Win::run failed");
    }

    Ok(())
}
