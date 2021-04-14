use std::fs::File;
use std::path::{Path, PathBuf};

use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, Align, Button, ButtonExt, ContainerExt, Grid, GridExt, Label, LabelExt,
    OrientableExt, PolicyType, ScrolledWindow, ScrolledWindowExt, Separator, WidgetExt, WrapMode,
};

use relm::{connect, Relm};

use wvr_data::config::project_config::ProjectConfig;

use crate::main_window::Msg;

use crate::main_window::MainWindow;

fn list_projects(projects_folder_path: &Path) -> Vec<PathBuf> {
    let mut is_project = false;

    if let Ok(path_list) = std::fs::read_dir(projects_folder_path) {
        for path in path_list {
            if let Ok(path) = path {
                if let Some(config_file_candidate) = path.file_name().to_str() {
                    if &config_file_candidate.to_lowercase() == "config.json" {
                        is_project = true;
                        break;
                    }
                }
            }
        }
    }

    if is_project {
        return vec![projects_folder_path.to_owned()];
    }

    let mut available_projects_list_widget = Vec::new();

    if let Ok(path_list) = std::fs::read_dir(projects_folder_path) {
        for path in path_list {
            if let Ok(path) = path {
                let path = path.path();
                if path.is_dir() {
                    available_projects_list_widget.extend(list_projects(&path));
                }
            }
        }
    }

    available_projects_list_widget
}

pub fn build_description_panel() -> gtk::Box {
    let description_panel = gtk::Box::new(Vertical, 4);

    let mut description_text = String::new();
    description_text
        .push_str("Wvr is an animation tool based on the compositon of filtered images/videos as well as programatically generated content.");

    let description_widget = Label::new(Some(&description_text));
    description_widget.set_hexpand(true);
    description_widget.set_line_wrap(true);
    description_widget.set_xalign(0.0);
    description_widget.set_halign(Align::Fill);

    description_panel.add(&description_widget);

    description_panel
}
pub fn build_available_projects_panel(relm: &Relm<MainWindow>) -> gtk::Box {
    let available_projects_panel = gtk::Box::new(Vertical, 4);

    let available_projects_label = Label::new(Some("Available projects"));
    available_projects_label.set_xalign(0.0);

    let available_projects_list_panel = Grid::new();

    available_projects_list_panel.set_hexpand(true);
    available_projects_list_panel.set_row_spacing(4);
    available_projects_list_panel.set_column_spacing(8);
    available_projects_list_panel.set_orientation(Vertical);

    let available_projects_list = list_projects(&wvr_data::get_data_path().join("projects"));

    let mut row_index = 0;
    for project_path in &available_projects_list {
        if let Some(project_path_as_str) = project_path.to_str() {
            let load_project_button = Button::new();
            load_project_button.set_label("Load");
            load_project_button.set_property_margin(4);

            let project_path_label = Label::new(Some(project_path_as_str));
            project_path_label.set_xalign(0.0);

            available_projects_list_panel.attach(&project_path_label, 0, row_index, 1, 1);
            available_projects_list_panel.attach(&load_project_button, 1, row_index, 1, 1);

            let project_path = project_path.clone();
            connect!(relm, load_project_button, connect_clicked(_), {
                let config_path = project_path.join("config.json");

                if let Ok(file) = File::open(&config_path) {
                    if let Ok(project_config) = serde_json::from_reader::<File, ProjectConfig>(file)
                    {
                        Some(Msg::OpenProject(project_path.clone(), project_config))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            row_index += 1;
        }
    }

    let available_projects_list_wrapper = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);
    available_projects_list_wrapper.set_size_request(0, 240);
    available_projects_list_wrapper.set_policy(PolicyType::Never, PolicyType::Automatic);
    available_projects_list_wrapper.set_hexpand(true);
    available_projects_list_wrapper.set_vexpand(true);
    available_projects_list_wrapper.add(&available_projects_list_panel);

    available_projects_panel.add(&available_projects_label);
    available_projects_panel.add(&Separator::new(Horizontal));
    available_projects_panel.add(&available_projects_list_wrapper);

    available_projects_panel
}

pub fn build_welcome_panel(relm: &Relm<MainWindow>) -> gtk::Box {
    let welcome_panel = gtk::Box::new(Vertical, 16);
    welcome_panel.set_property_margin(16);
    //welcome_panel.add(&build_description_panel());
    welcome_panel.add(&build_available_projects_panel(relm));

    welcome_panel
}
