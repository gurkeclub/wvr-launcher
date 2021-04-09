use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, ContainerExt, EditableSignals, Entry, EntryExt, FileChooserAction,
    FileChooserButton, FileChooserButtonExt, FileChooserExt, Label, LabelExt, RadioButton,
    SpinButton, SpinButtonExt, StateFlags, ToggleButtonExt, WidgetExt,
};

use relm::{connect, Relm};
use wvr_data::config::project_config::{InputConfig, Speed};

use super::InputConfigView;
use super::InputConfigViewModel;
use super::InputConfigViewMsg;

pub fn build_video_view(
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

    if let InputConfig::Video {
        path,
        width,
        height,
        speed,
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

        let video_path_row = gtk::Box::new(Horizontal, 8);
        video_path_row.set_property_margin(8);

        let video_path_label = Label::new(Some("Path: "));
        video_path_label.set_xalign(0.0);
        video_path_label.set_size_request(48, 0);

        let video_path = FileChooserButton::new("Select video file", FileChooserAction::Open);

        let absolute_path = project_path.join(&path);
        let absolute_path = if absolute_path.exists() {
            Some(absolute_path)
        } else if let Ok(absolute_path) = PathBuf::from_str(&path) {
            Some(absolute_path)
        } else {
            None
        };

        if let Some(absolute_path) = absolute_path {
            video_path.set_filename(&absolute_path);
        }

        video_path.set_hexpand(true);
        connect!(
            relm,
            video_path,
            connect_file_set(val),
            if let Some(path) = val.get_filename() {
                Some(InputConfigViewMsg::SetPath(
                    path.to_str().unwrap().to_string(),
                ))
            } else {
                None
            }
        );

        video_path_row.add(&video_path_label);
        video_path_row.add(&video_path);

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

        // Speed row creation
        let speed_row = gtk::Box::new(Horizontal, 8);
        speed_row.set_property_margin(8);

        let padding = gtk::Box::new(Horizontal, 0);
        padding.set_hexpand(true);

        let speed_type_button_fps = RadioButton::with_label("Fps");
        let speed_type_button_beats =
            RadioButton::with_label_from_widget(&speed_type_button_fps, "Beats");

        let (speed_is_bpm, speed_value) = match speed {
            Speed::Beats(speed) => (true, speed),
            Speed::Fps(speed) => (false, speed),
        };

        if speed_is_bpm {
            speed_type_button_beats.set_active(true);
        } else {
            speed_type_button_fps.set_active(true);
        }

        connect!(
            relm,
            speed_type_button_beats,
            connect_property_active_notify(val),
            Some(InputConfigViewMsg::SetSpeedIsBpm(val.get_active()))
        );
        connect!(
            relm,
            speed_type_button_fps,
            connect_property_active_notify(val),
            Some(InputConfigViewMsg::SetSpeedIsBpm(!val.get_active()))
        );

        let speed_spin_button = SpinButton::new(
            Some(&Adjustment::new(
                *speed_value as f64,
                0.0,
                8192.0,
                0.01,
                1.0,
                10.0,
            )),
            1.0,
            2,
        );
        connect!(
            relm,
            speed_spin_button,
            connect_changed(val),
            if let Ok(value) = val.get_text().as_str().replace(',', ".").parse::<f64>() {
                Some(InputConfigViewMsg::SetSpeed(value))
            } else {
                None
            }
        );

        speed_row.add(&Label::new(Some("Speed: ")));
        speed_row.add(&padding);
        speed_row.add(&speed_type_button_fps);
        speed_row.add(&speed_type_button_beats);
        speed_row.add(&speed_spin_button);

        root.add(&name_row);
        root.add(&video_path_row);
        root.add(&resolution_row);
        root.add(&speed_row);
    } else {
        panic!("Cannot build a video config view from {:?}", model.config);
    }

    root
}
