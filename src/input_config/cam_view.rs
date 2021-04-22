use uuid::Uuid;

use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, ContainerExt, EditableSignals, Entry, EntryExt, Label, LabelExt, SpinButton,
    StateFlags, WidgetExt,
};

use relm::{connect, Relm};

use wvr_data::config::project_config::InputConfig;

use super::InputConfigViewMsg;
use crate::config_panel::{msg::ConfigPanelMsg, view::ConfigPanel};

pub fn build_cam_view(
    relm: &Relm<ConfigPanel>,
    id: Uuid,
    name: &str,
    config: &InputConfig,
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

    if let InputConfig::Cam {
        path,
        width,
        height,
    } = config
    {
        let name_row = gtk::Box::new(Horizontal, 8);
        name_row.set_property_margin(8);

        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);
        name_label.set_size_request(48, 0);

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

        name_row.add(&name_label);
        name_row.add(&name_entry);

        let cam_path_row = gtk::Box::new(Horizontal, 8);
        cam_path_row.set_property_margin(8);

        let cam_path_label = Label::new(Some("Path: "));
        cam_path_label.set_xalign(0.0);
        cam_path_label.set_size_request(48, 0);

        let cam_path = Entry::new();
        cam_path.set_text(&path);
        cam_path.set_hexpand(true);
        connect!(
            relm,
            cam_path,
            connect_changed(val),
            ConfigPanelMsg::UpdateInput(
                id,
                InputConfigViewMsg::SetPath(val.get_text().to_string())
            )
        );

        cam_path_row.add(&cam_path_label);
        cam_path_row.add(&cam_path);

        // Resolution row creation
        let resolution_row = gtk::Box::new(Horizontal, 8);
        resolution_row.set_property_margin(8);

        let resolution_label = Label::new(Some("Size: "));
        resolution_label.set_xalign(0.0);
        resolution_label.set_size_request(48, 0);

        let padding = gtk::Box::new(Horizontal, 0);
        padding.set_hexpand(true);

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

        resolution_row.add(&resolution_label);
        resolution_row.add(&width_spin_button);
        resolution_row.add(&Label::new(Some("x")));
        resolution_row.add(&height_spin_button);

        root.add(&name_row);
        root.add(&cam_path_row);
        root.add(&resolution_row);

        root
    } else {
        panic!("Cannot build a camera config view from {:?}", config);
    }
}
