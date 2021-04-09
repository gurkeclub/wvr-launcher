use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, ContainerExt, EditableSignals, Entry, EntryExt, FileChooserAction,
    FileChooserButton, FileChooserButtonExt, FileChooserExt, Label, LabelExt, SpinButton,
    SpinButtonExt, StateFlags, WidgetExt,
};

use relm::{connect, Relm};
use wvr_data::config::project_config::InputConfig;

use super::InputConfigView;
use super::InputConfigViewModel;
use super::InputConfigViewMsg;

pub fn build_picture_view(
    relm: &Relm<InputConfigView>,
    project_path: &Path,
    model: &InputConfigViewModel,
) -> gtk::Box {
    let root = gtk::Box::new(Vertical, 0);
    root.override_background_color(
        StateFlags::NORMAL,
        Some(&RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.125,
        }),
    );

    if let InputConfig::Picture {
        path,
        width,
        height,
    } = &model.config
    {
        let name_row = gtk::Box::new(Horizontal, 8);
        name_row.set_property_margin(8);

        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);
        name_label.set_size_request(48, 0);

        let name_entry = Entry::new();
        name_entry.set_text(&model.name);
        name_entry.set_hexpand(true);
        connect!(
            relm,
            name_entry,
            connect_changed(val),
            Some(InputConfigViewMsg::SetName(val.get_text().to_string()))
        );

        name_row.add(&name_label);
        name_row.add(&name_entry);

        let picture_path_row = gtk::Box::new(Horizontal, 8);
        picture_path_row.set_property_margin(8);

        let picture_path_label = Label::new(Some("Path: "));
        picture_path_label.set_xalign(0.0);
        picture_path_label.set_size_request(48, 0);

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
                Some(InputConfigViewMsg::SetPath(
                    path.to_str().unwrap().to_string(),
                ))
            } else {
                None
            }
        );

        picture_path_row.add(&picture_path_label);
        picture_path_row.add(&picture_path);

        // Resolution row creation
        let resolution_row = gtk::Box::new(Horizontal, 8);
        resolution_row.set_property_margin(8);

        let resolution_label = Label::new(Some("Size: "));
        resolution_label.set_xalign(0.0);
        resolution_label.set_size_request(48, 0);

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
                Some(InputConfigViewMsg::SetWidth(value as i64))
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
                Some(InputConfigViewMsg::SetHeight(value as i64))
            } else {
                None
            }
        );

        resolution_row.add(&resolution_label);
        resolution_row.add(&width_spin_button);
        resolution_row.add(&Label::new(Some("x")));
        resolution_row.add(&height_spin_button);

        root.add(&name_row);
        root.add(&picture_path_row);
        root.add(&resolution_row);
    } else {
        panic!("Cannot build a picture config view from {:?}", model.config);
    }

    root
}
