use std::fs::File;
use std::path::PathBuf;

use nfd2::Response;

use relm_derive::Msg;

use glib::object::ObjectExt;
use gtk::prelude::*;

use gtk::Orientation::Vertical;
use gtk::{
    AccelFlags, AccelGroup, ContainerExt, GtkWindowExt, Inhibit, Menu, MenuBar, MenuItem, Settings,
    WidgetExt, Window, WindowType,
};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};

use wvr_data::config::project_config::ProjectConfig;

use crate::main_panel::MainPanel;

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
    LoadProject(PathBuf, ProjectConfig),
    SaveProject,
    Quit,
}
pub struct Model {}

pub struct MainWindow {
    model: Model,

    window: Window,
    panel_container: gtk::Box,
    panel: Option<Component<MainPanel>>,

    relm: Relm<Self>,
}

impl MainWindow {}

impl Update for MainWindow {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Msg) {
        println!("{:?}", event);

        match event {
            Msg::LoadProject(project_path, project_config) => {
                for children in &self.panel_container.get_children() {
                    self.panel_container.remove(children);
                }
                self.panel = Some(
                    self.panel_container
                        .add_widget::<MainPanel>((project_path, project_config)),
                );
                self.panel_container.show_all();
            }
            Msg::SaveProject => {
                if let Some(panel) = &self.panel {
                    panel.emit(crate::Msg::Save);
                }
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for MainWindow {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let settings = Settings::get_default().unwrap();
        settings
            .set_property("gtk-application-prefer-dark-theme", &true)
            .unwrap();

        let window = Window::new(WindowType::Toplevel);

        window.hide();

        window.set_size_request(640, 640);
        window.set_title("wvr");
        window.set_position(gtk::WindowPosition::Center);

        let v_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let accel_group = AccelGroup::new();
        window.add_accel_group(&accel_group);

        let menu_bar = MenuBar::new();

        let file = MenuItem::with_label("File");
        let file_menu = Menu::new();
        let open_menu_item = MenuItem::with_label("Open");
        let save_menu_item = MenuItem::with_label("Save");
        let quit = MenuItem::with_label("Quit");

        file_menu.append(&open_menu_item);
        file_menu.append(&save_menu_item);
        file_menu.append(&quit);

        file.set_submenu(Some(&file_menu));
        menu_bar.append(&file);

        {
            let window = window.clone();
            connect!(relm, quit, connect_activate(_), {
                window.close();
            });
        }

        // `Primary` is `Ctrl` on Windows and Linux, and `command` on macOS
        // It isn't available directly through gdk::ModifierType, since it has
        // different values on different platforms.
        let (key, modifier) = gtk::accelerator_parse("<Primary>Q");
        quit.add_accelerator("activate", &accel_group, key, modifier, AccelFlags::VISIBLE);

        connect!(
            relm,
            open_menu_item,
            connect_activate(_),
            if let Some(project) = get_config() {
                Some(Msg::LoadProject(project.0, project.1))
            } else {
                None
            }
        );

        connect!(relm, save_menu_item, connect_activate(_), Msg::SaveProject);

        v_box.pack_start(&menu_bar, false, false, 0);

        let panel_container = gtk::Box::new(Vertical, 0);
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
            model,

            window,
            panel_container,
            panel: None,

            relm: relm.clone(),
        }
    }
}
