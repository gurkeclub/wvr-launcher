use uuid::Uuid;

use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{ContainerExt, EditableSignals, Entry, EntryExt, Label, LabelExt, StateFlags, WidgetExt};

use relm::{connect, Relm};
use wvr_data::config::project_config::InputConfig;

use crate::config_panel::{msg::ConfigPanelMsg, view::ConfigPanel};

use super::InputConfigViewMsg;

pub fn build_midi_view(
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

    if let InputConfig::Midi { name: pattern } = config {
        let name_row = gtk::Box::new(Horizontal, 8);
        name_row.set_property_margin(8);

        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);
        name_label.set_size_request(48, 0);

        let name_entry = Entry::new();
        name_entry.set_text(pattern);
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

        // Create Midi ID pattern row
        let id_pattern_row = gtk::Box::new(Horizontal, 8);
        id_pattern_row.set_property_margin(8);

        let id_pattern_label = Label::new(Some("Pattern: "));
        id_pattern_label.set_xalign(0.0);
        id_pattern_label.set_size_request(48, 0);

        let id_pattern = Entry::new();
        id_pattern.set_text(&name);
        id_pattern.set_hexpand(true);
        connect!(
            relm,
            id_pattern,
            connect_changed(val),
            ConfigPanelMsg::UpdateInput(
                id,
                InputConfigViewMsg::SetPath(val.get_text().to_string())
            )
        );

        id_pattern_row.add(&id_pattern_label);
        id_pattern_row.add(&id_pattern);

        root.add(&name_row);
        root.add(&id_pattern_row);

        root
    } else {
        panic!("Cannot build a camera config view from {:?}", config);
    }
}
