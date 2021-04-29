use uuid::Uuid;

use gtk::{
    EditableSignals, Entry, EntryExt, GridExt, Label, LabelExt, OrientableExt, Orientation,
    WidgetExt,
};

use relm::{connect, Relm};
use wvr_data::config::project_config::InputConfig;

use crate::config_panel::{msg::ConfigPanelMsg, view::ConfigPanel};

use super::InputConfigViewMsg;

pub fn build_midi_view(
    relm: &Relm<ConfigPanel>,
    id: Uuid,
    name: &str,
    config: &InputConfig,
) -> gtk::Grid {
    let root = gtk::Grid::new();
    root.set_row_spacing(4);
    root.set_column_spacing(4);
    root.set_orientation(Orientation::Vertical);

    if let InputConfig::Midi { name: pattern } = config {
        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);

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

        root.attach(&name_label, 0, 0, 1, 1);
        root.attach(&name_entry, 1, 0, 1, 1);

        // Create Midi ID pattern row
        let id_pattern_label = Label::new(Some("Pattern: "));
        id_pattern_label.set_xalign(0.0);

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

        root.attach(&id_pattern_label, 0, 1, 1, 1);
        root.attach(&id_pattern, 1, 1, 1, 1);

        root
    } else {
        panic!("Cannot build a camera config view from {:?}", config);
    }
}
