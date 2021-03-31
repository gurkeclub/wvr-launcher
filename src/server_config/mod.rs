use gtk::Orientation::{Horizontal, Vertical};
use gtk::{Adjustment, ContainerExt, EditableSignals, Entry, EntryExt, Label, SpinButton, SpinButtonExt, Switch, SwitchExt, WidgetExt};

use relm::{connect, Relm};

use crate::Msg;

use wvr_data::config::server_config::ServerConfig;

pub fn build_view(relm: &Relm<crate::Win>, server_config: &ServerConfig) -> gtk::Box {
    let view_config_container = gtk::Box::new(Vertical, 2);
    view_config_container.set_property_margin(4);

    // Server IP row creation
    let ip_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let ip_entry = Entry::new();
    ip_entry.set_text(&server_config.ip);
    connect!(relm, ip_entry, connect_changed(val), Some(Msg::SetServerIp(val.get_text().to_string())));

    ip_row.add(&Label::new(Some("Server binding IP: ")));
    ip_row.add(&padding);
    ip_row.add(&ip_entry);

    // Server port row creation
    let port_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let width_spin_button = SpinButton::new(Some(&Adjustment::new(server_config.port as f64, 0.0, 8192.0, 1.0, 10.0, 10.0)), 1.0, 0);
    connect!(relm, width_spin_button, connect_changed(val), Some(Msg::SetServerPort(val.get_value() as i64)));

    port_row.add(&Label::new(Some("Listening port: ")));
    port_row.add(&padding);
    port_row.add(&width_spin_button);

    // Enable server row creation
    let enable_server_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let enable_server_switch = Switch::new();
    enable_server_switch.set_state(server_config.enable);
    connect!(relm, enable_server_switch, connect_property_active_notify(val), Some(Msg::SetServerEnabled(val.get_state())));

    enable_server_row.add(&Label::new(Some("Enable server: ")));
    enable_server_row.add(&padding);
    enable_server_row.add(&enable_server_switch);

    view_config_container.add(&ip_row);
    view_config_container.add(&port_row);
    view_config_container.add(&enable_server_row);

    view_config_container
}