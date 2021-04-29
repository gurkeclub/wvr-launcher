use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use uuid::Uuid;

use gtk::{
    Adjustment, ContainerExt, EditableSignals, Entry, EntryExt, FileChooserAction,
    FileChooserButton, FileChooserButtonExt, FileChooserExt, GridExt, Label, LabelExt,
    OrientableExt, Orientation, SpinButton, WidgetExt,
};

use relm::{connect, Relm};
use wvr_data::config::project_config::InputConfig;

use crate::config_panel::{msg::ConfigPanelMsg, view::ConfigPanel};

use super::InputConfigViewMsg;

pub fn build_picture_view(
    relm: &Relm<ConfigPanel>,
    project_path: &Path,
    id: Uuid,
    name: &str,
    config: &InputConfig,
) -> gtk::Grid {
    let root = gtk::Grid::new();
    root.set_row_spacing(4);
    root.set_column_spacing(4);
    root.set_orientation(Orientation::Vertical);

    if let InputConfig::Picture {
        path,
        width,
        height,
    } = &config
    {
        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);

        let name_entry = Entry::new();
        name_entry.set_text(name);
        name_entry.set_hexpand(true);
        connect!(
            relm,
            name_entry,
            connect_changed(val),
            ConfigPanelMsg::UpdateInput(
                id,
                InputConfigViewMsg::SetName(val.get_text().to_string())
            )
        );

        root.attach(&name_label, 0, 0, 1, 1);
        root.attach(&name_entry, 1, 0, 1, 1);

        let picture_path_label = Label::new(Some("Path: "));
        picture_path_label.set_xalign(0.0);

        let picture_path = FileChooserButton::new("Select picture file", FileChooserAction::Open);

        let absolute_path = project_path.join(&path);
        let absolute_path = if absolute_path.exists() {
            Some(absolute_path)
        } else if let Ok(absolute_path) = PathBuf::from_str(&path) {
            Some(absolute_path)
        } else {
            None
        };

        if let Some(absolute_path) = absolute_path {
            picture_path.set_filename(&absolute_path);
        }

        picture_path.set_hexpand(true);
        connect!(
            relm,
            picture_path,
            connect_file_set(val),
            if let Some(path) = val.get_filename() {
                Some(ConfigPanelMsg::UpdateInput(
                    id,
                    InputConfigViewMsg::SetPath(path.to_str().unwrap().to_string()),
                ))
            } else {
                None
            }
        );

        root.attach(&picture_path_label, 0, 1, 1, 1);
        root.attach(&picture_path, 1, 1, 1, 1);

        // Resolution row creation
        let resolution_label = Label::new(Some("Size: "));
        resolution_label.set_xalign(0.0);

        let resolution_row = gtk::Box::new(Orientation::Horizontal, 4);

        let width_spin_button = SpinButton::new(
            Some(&Adjustment::new(
                *width as f64,
                0.0,
                8192.0,
                1.0,
                10.0,
                10.0,
            )),
            1.0,
            0,
        );
        width_spin_button.set_hexpand(true);
        connect!(
            relm,
            width_spin_button,
            connect_changed(val),
            if let Ok(value) = val.get_text().as_str().replace(',', ".").parse::<f64>() {
                Some(ConfigPanelMsg::UpdateInput(
                    id,
                    InputConfigViewMsg::SetWidth(value as i64),
                ))
            } else {
                None
            }
        );

        let height_spin_button = SpinButton::new(
            Some(&Adjustment::new(
                *height as f64,
                0.0,
                8192.0,
                1.0,
                10.0,
                10.0,
            )),
            1.0,
            0,
        );
        height_spin_button.set_hexpand(true);
        connect!(
            relm,
            height_spin_button,
            connect_changed(val),
            if let Ok(value) = val.get_text().as_str().replace(',', ".").parse::<f64>() {
                Some(ConfigPanelMsg::UpdateInput(
                    id,
                    InputConfigViewMsg::SetHeight(value as i64),
                ))
            } else {
                None
            }
        );
        resolution_row.add(&width_spin_button);
        resolution_row.add(&Label::new(Some("x")));
        resolution_row.add(&height_spin_button);

        root.attach(&resolution_label, 0, 2, 1, 1);
        root.attach(&resolution_row, 1, 2, 1, 1);

        root
    } else {
        panic!("Cannot build a picture config view from {:?}", config);
    }
}
