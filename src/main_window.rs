use std::path::PathBuf;

use relm::connect;
use relm_derive::Msg;

use glib::object::ObjectExt;
use gtk::prelude::*;

use gtk::Orientation;
use gtk::{
    AccelFlags, AccelGroup, ButtonsType, ContainerExt, DialogBuilder, DialogExt, Entry, EntryExt,
    GtkWindowExt, Inhibit, Menu, MenuBar, MenuItem, MessageDialogBuilder, MessageType,
    ResponseType, Settings, WidgetExt, Window, WindowPosition, WindowType,
};

use relm::{Component, ContainerWidget, Relm, Update, Widget};

use wvr_data::config::project_config::ProjectConfig;

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;
use crate::welcome_panel;

#[derive(Msg, Debug)]
pub enum Msg {
    UpdateConfig(ProjectConfig),
    NewProject,
    OpenProject(PathBuf, ProjectConfig),
    SaveProject,
    ToggleDarkMode,
    Quit,
}
pub struct Model {
    project_path: Option<PathBuf>,
    project_config: Option<ProjectConfig>,
    dark_mode: bool,
}

pub struct MainWindow {
    model: Model,
    window: Window,

    config_panel_container: gtk::Box,
    config_panel: Option<Component<ConfigPanel>>,

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
                        } else if let Some(project_config) =
                            crate::utils::create_project(&new_project_path)
                        {
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
        Model {
            project_path: None,
            project_config: None,
            dark_mode: true,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ToggleDarkMode => {
                self.model.dark_mode = !self.model.dark_mode;

                let settings = Settings::get_default().unwrap();
                settings
                    .set_property("gtk-application-prefer-dark-theme", &self.model.dark_mode)
                    .unwrap();
            }
            Msg::UpdateConfig(project_config) => {
                self.model.project_config = Some(project_config);
            }
            Msg::OpenProject(project_path, project_config) => {
                self.model.project_path = Some(project_path.clone());
                self.model.project_config = Some(project_config.clone());

                for children in &self.config_panel_container.get_children() {
                    self.config_panel_container.remove(children);
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

                self.config_panel = Some(self.config_panel_container.add_widget::<ConfigPanel>((
                    self.relm.clone(),
                    project_path,
                    project_config,
                )));

                self.root().show_all();
            }
            Msg::SaveProject => {
                if let Some(panel) = &self.config_panel {
                    panel.emit(ConfigPanelMsg::Save);
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

    let file_button = MenuItem::with_label("File");
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

    file_button.set_submenu(Some(&file_menu));
    file_menu.append(&new_menu_item);
    file_menu.append(&open_menu_item);
    file_menu.append(&save_menu_item);
    file_menu.append(&quit);

    let view_button = MenuItem::with_label("View");
    let view_menu = Menu::new();

    let dark_mode_button = MenuItem::with_label("Toggle dark theme");
    let (key, modifier) = gtk::accelerator_parse("<Primary>D");
    dark_mode_button.add_accelerator("activate", accel_group, key, modifier, AccelFlags::VISIBLE);

    view_button.set_submenu(Some(&view_menu));
    view_menu.append(&dark_mode_button);

    menu_bar.append(&file_button);
    menu_bar.append(&view_button);

    connect!(relm, new_menu_item, connect_activate(_), Msg::NewProject);

    connect!(
        relm,
        open_menu_item,
        connect_activate(_),
        if let Some(project) = crate::utils::get_config() {
            Some(Msg::OpenProject(project.0, project.1))
        } else {
            None
        }
    );

    connect!(relm, save_menu_item, connect_activate(_), Msg::SaveProject);

    connect!(
        relm,
        dark_mode_button,
        connect_activate(_),
        Msg::ToggleDarkMode
    );

    connect!(relm, quit, connect_activate(_), Msg::Quit);

    menu_bar
}

impl Widget for MainWindow {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let settings = Settings::get_default().unwrap();
        settings
            .set_property("gtk-application-prefer-dark-theme", &model.dark_mode)
            .unwrap();

        let provider = gtk::CssProvider::new();
        // Load the CSS file
        let style = include_bytes!("../res/style.css");
        provider.load_from_data(style).expect("Failed to load CSS");
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let window = Window::new(WindowType::Toplevel);
        window.hide();
        window.set_title("wvr");
        window.set_position(gtk::WindowPosition::Center);
        window.maximize();

        if let Err(e) = wvr::utils::init_wvr_data_directory() {
            let error_message = MessageDialogBuilder::new()
                .title("Error initializing wvr files")
                .text(&format!("{:?}", e))
                .message_type(MessageType::Error)
                .window_position(WindowPosition::Center)
                .buttons(ButtonsType::Ok)
                .attached_to(&window)
                .modal(true)
                .build();
            error_message.run();
            error_message.close();
        }

        let accel_group = AccelGroup::new();
        window.add_accel_group(&accel_group);

        let v_box = gtk::Box::new(Orientation::Vertical, 0);

        v_box.pack_start(&build_menu_bar(relm, &accel_group), false, false, 0);

        let config_panel_container = gtk::Box::new(Orientation::Vertical, 0);

        config_panel_container.add(&welcome_panel::build_welcome_panel(relm));

        v_box.pack_start(&config_panel_container, true, true, 0);

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

            config_panel_container,
            config_panel: None,

            relm: relm.clone(),
        }
    }
}
