use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use nfd2::Response;

use relm::connect;
use relm_derive::Msg;

use glib::object::ObjectExt;
use gtk::prelude::*;

use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    AccelFlags, AccelGroup, ButtonsType, ContainerExt, DialogBuilder, DialogExt, Entry, EntryExt,
    GtkWindowExt, Inhibit, Menu, MenuBar, MenuItem, MessageDialogBuilder, MessageType,
    ResponseType, Settings, WidgetExt, Window, WindowPosition, WindowType,
};

use relm::{Component, ContainerWidget, Relm, Update, Widget};

use wvr_data::config::project_config::{
    BufferPrecision, FilterMode, ProjectConfig, RenderStageConfig, SampledInput, ViewConfig,
};
use wvr_data::config::server_config::ServerConfig;

use crate::main_panel::MainPanel;
use crate::welcome_panel;

fn create_project(project_config_path: &Path) -> Option<ProjectConfig> {
    std::fs::create_dir_all(&project_config_path).unwrap();
    std::fs::create_dir_all(&project_config_path.join("filters")).unwrap();

    let patterns_stage = RenderStageConfig {
        name: "Patterns".to_owned(),
        filter: "dot_grid".to_owned(),
        filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 1.0, 1.0),
        inputs: HashMap::new(),
        variables: HashMap::new(),
        precision: BufferPrecision::F32,
    };

    let mut final_stage_input_list = HashMap::new();
    final_stage_input_list.insert(
        "iChannel0".to_owned(),
        SampledInput::Linear("Patterns".to_owned()),
    );

    let final_stage = RenderStageConfig {
        name: "FinalStage".to_owned(),
        filter: "copy_input".to_owned(),
        filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 1.0, 1.0),
        inputs: final_stage_input_list,
        variables: HashMap::new(),
        precision: BufferPrecision::F32,
    };

    let project_config = ProjectConfig {
        bpm: 89.0,
        view: ViewConfig {
            width: 640,
            height: 480,
            fullscreen: false,
            dynamic: true,
            vsync: true,
            screenshot: false,
            screenshot_path: PathBuf::from("output/"),
            target_fps: 60.0,
            locked_speed: false,
        },
        server: ServerConfig {
            ip: "localhost".to_owned(),
            port: 3000,
            enable: false,
        },
        inputs: HashMap::new(),
        render_chain: vec![patterns_stage],
        final_stage,
    };
    if let Ok(mut project_config_file) =
        std::fs::File::create(&project_config_path.join("config.json"))
    {
        project_config_file
            .write_all(
                &serde_json::ser::to_string_pretty(&project_config)
                    .unwrap()
                    .into_bytes(),
            )
            .unwrap();

        Some(project_config)
    } else {
        None
    }
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
#[derive(Msg, Debug)]
pub enum Msg {
    NewProject,
    OpenProject(PathBuf, ProjectConfig),
    SaveProject,
    Quit,
}
pub struct Model {}

pub struct MainWindow {
    window: Window,
    panel_container: gtk::Box,
    panel: Option<Component<MainPanel>>,

    relm: Relm<Self>,
}

impl MainWindow {
    pub fn open_new_project_dialog(&self) -> Option<(PathBuf, ProjectConfig)> {
        let new_project_dialog = DialogBuilder::new()
            .title("New wvr project")
            .attached_to(&self.window)
            .window_position(WindowPosition::Center)
            .modal(true)
            .build();
        new_project_dialog.set_response_sensitive(ResponseType::Ok, false);

        let new_project_name_entry = Entry::new();
        {
            let new_project_dialog = new_project_dialog.clone();
            connect!(self.relm, new_project_name_entry, connect_changed(val), {
                new_project_dialog.set_response_sensitive(
                    ResponseType::Ok,
                    !val.get_text().to_string().trim().is_empty(),
                );
            });
        }

        new_project_dialog
            .get_content_area()
            .add(&new_project_name_entry);
        new_project_dialog
            .add_button("Ok", ResponseType::Ok)
            .set_hexpand(true);
        new_project_dialog.add_button("Cancel", ResponseType::Cancel);

        new_project_dialog.get_content_area().show_all();

        let mut created_project = None;
        while created_project.is_none() {
            match new_project_dialog.run() {
                ResponseType::Ok => {
                    let new_project_name_candidate = new_project_name_entry
                        .get_text()
                        .to_string()
                        .trim()
                        .to_string();

                    if !new_project_name_candidate.is_empty() {
                        let new_project_path = wvr_data::get_data_path()
                            .join("projects")
                            .join(&new_project_name_candidate);
                        if new_project_path.exists() {
                            let error_message = MessageDialogBuilder::new()
                                .title("Error: project exists")
                                .text("A project using the same name already exists")
                                .message_type(MessageType::Error)
                                .window_position(WindowPosition::Center)
                                .buttons(ButtonsType::Ok)
                                .attached_to(&self.window)
                                .modal(true)
                                .build();
                            error_message.run();
                            error_message.close();
                        } else if let Some(project_config) = create_project(&new_project_path) {
                            created_project = Some((new_project_path, project_config));
                        }
                    }
                }
                ResponseType::Cancel => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        new_project_dialog.close();

        created_project
    }
}

impl Update for MainWindow {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::OpenProject(project_path, project_config) => {
                for children in &self.panel_container.get_children() {
                    self.panel_container.remove(children);
                }
                self.window.set_title(&format!(
                    "wvr://{:}",
                    project_path
                        .ancestors()
                        .next()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                ));
                self.panel = Some(
                    self.panel_container
                        .add_widget::<MainPanel>((project_path, project_config)),
                );
                self.panel_container.show_all();
            }
            Msg::SaveProject => {
                if let Some(panel) = &self.panel {
                    panel.emit(crate::main_panel::Msg::Save);
                }
            }

            Msg::NewProject => {
                if let Some((new_project_path, new_project_config)) = self.open_new_project_dialog()
                {
                    self.relm
                        .stream()
                        .emit(Msg::OpenProject(new_project_path, new_project_config));
                }
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

fn build_menu_bar(relm: &Relm<MainWindow>, accel_group: &AccelGroup) -> MenuBar {
    let menu_bar = MenuBar::new();

    let file = MenuItem::with_label("File");
    let file_menu = Menu::new();

    let new_menu_item = MenuItem::with_label("New");
    let (key, modifier) = gtk::accelerator_parse("<Primary>N");
    new_menu_item.add_accelerator("activate", accel_group, key, modifier, AccelFlags::VISIBLE);

    let open_menu_item = MenuItem::with_label("Open");
    let (key, modifier) = gtk::accelerator_parse("<Primary>O");
    open_menu_item.add_accelerator("activate", accel_group, key, modifier, AccelFlags::VISIBLE);

    let save_menu_item = MenuItem::with_label("Save");
    let (key, modifier) = gtk::accelerator_parse("<Primary>S");
    save_menu_item.add_accelerator("activate", accel_group, key, modifier, AccelFlags::VISIBLE);

    let quit = MenuItem::with_label("Quit");
    let (key, modifier) = gtk::accelerator_parse("<Primary>Q");
    quit.add_accelerator("activate", accel_group, key, modifier, AccelFlags::VISIBLE);

    file_menu.append(&new_menu_item);
    file_menu.append(&open_menu_item);
    file_menu.append(&save_menu_item);
    file_menu.append(&quit);

    file.set_submenu(Some(&file_menu));
    menu_bar.append(&file);

    connect!(relm, new_menu_item, connect_activate(_), Msg::NewProject);

    connect!(
        relm,
        open_menu_item,
        connect_activate(_),
        if let Some(project) = get_config() {
            Some(Msg::OpenProject(project.0, project.1))
        } else {
            None
        }
    );

    connect!(relm, save_menu_item, connect_activate(_), Msg::SaveProject);

    connect!(relm, quit, connect_activate(_), Msg::Quit);

    menu_bar
}

impl Widget for MainWindow {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _: Self::Model) -> Self {
        let settings = Settings::get_default().unwrap();
        settings
            .set_property("gtk-application-prefer-dark-theme", &true)
            .unwrap();

        let window = Window::new(WindowType::Toplevel);
        window.hide();
        window.set_title("wvr");
        window.set_position(gtk::WindowPosition::Center);

        let accel_group = AccelGroup::new();
        window.add_accel_group(&accel_group);

        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        v_box.pack_start(&build_menu_bar(relm, &accel_group), false, false, 0);

        let panel_container = gtk::Box::new(Vertical, 0);

        panel_container.add(&welcome_panel::build_welcome_panel(relm));
        v_box.add(&panel_container);

        //v_box.pack_start(&label, true, true, 0);

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        window.add(&v_box);

        window.show_all();

        MainWindow {
            window,
            panel_container,
            panel: None,

            relm: relm.clone(),
        }
    }
}
