use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, ContainerExt, EditableSignals, EntryExt, Label, SpinButton, Switch, SwitchExt,
    WidgetExt,
};

use relm::{connect, Relm};

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;

use wvr_data::config::project_config::ViewConfig;

pub fn build_view(relm: &Relm<ConfigPanel>, bpm: f64, view_config: &ViewConfig) -> gtk::Box {
    let view_config_container = gtk::Box::new(Vertical, 2);
    view_config_container.set_property_margin(8);

    // Fullscreen activation row creation
    let fullscreen_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let fullscreen_switch = Switch::new();
    fullscreen_switch.set_state(view_config.fullscreen);
    connect!(
        relm,
        fullscreen_switch,
        connect_property_active_notify(val),
        Some(ConfigPanelMsg::SetFullscreen(val.get_state()))
    );

    fullscreen_row.add(&Label::new(Some("Enable fullscreen: ")));
    fullscreen_row.add(&padding);
    fullscreen_row.add(&fullscreen_switch);

    // Dynamic resolution row creation
    let dynamic_size_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let dynamic_size_switch = Switch::new();
    dynamic_size_switch.set_state(view_config.dynamic);
    connect!(
        relm,
        dynamic_size_switch,
        connect_property_active_notify(val),
        Some(ConfigPanelMsg::SetDynamicResolution(val.get_state()))
    );

    dynamic_size_row.add(&Label::new(Some("Dynamic Resolution: ")));
    dynamic_size_row.add(&padding);
    dynamic_size_row.add(&dynamic_size_switch);

    // VSync activation row creation
    let vsync_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let vsync_switch = Switch::new();
    vsync_switch.set_state(view_config.vsync);
    connect!(
        relm,
        vsync_switch,
        connect_property_active_notify(val),
        Some(ConfigPanelMsg::SetVSync(val.get_state()))
    );

    vsync_row.add(&Label::new(Some("VSync: ")));
    vsync_row.add(&padding);
    vsync_row.add(&vsync_switch);

    // Screenshot activation row creation
    let screenshot_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let screenshot_switch = Switch::new();
    screenshot_switch.set_state(view_config.screenshot);
    connect!(
        relm,
        screenshot_switch,
        connect_property_active_notify(val),
        Some(ConfigPanelMsg::SetScreenshot(val.get_state()))
    );

    screenshot_row.add(&Label::new(Some("Enable frame recording: ")));
    screenshot_row.add(&padding);
    screenshot_row.add(&screenshot_switch);

    // Locked speed activation row creation
    let locked_speed_row = gtk::Box::new(Horizontal, 8);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let target_fps_spin_button = SpinButton::new(
        Some(&Adjustment::new(
            view_config.target_fps as f64,
            1.0,
            240.0,
            0.01,
            1.0,
            1.0,
        )),
        1.0,
        2,
    );
    connect!(
        relm,
        target_fps_spin_button,
        connect_changed(val),
        if let Ok(value) = val.get_text().as_str().replace(',', ".").parse::<f64>() {
            Some(ConfigPanelMsg::SetTargetFps(value))
        } else {
            None
        }
    );

    let locked_speed_switch = Switch::new();
    locked_speed_switch.set_state(view_config.locked_speed);
    connect!(
        relm,
        locked_speed_switch,
        connect_property_active_notify(val),
        Some(ConfigPanelMsg::SetLockedSpeed(val.get_state()))
    );

    locked_speed_row.add(&Label::new(Some("Lock Framerate: ")));
    locked_speed_row.add(&padding);
    locked_speed_row.add(&locked_speed_switch);
    locked_speed_row.add(&target_fps_spin_button);

    view_config_container.add(&locked_speed_row);
    view_config_container.add(&fullscreen_row);
    view_config_container.add(&dynamic_size_row);
    view_config_container.add(&vsync_row);
    view_config_container.add(&screenshot_row);

    view_config_container
}
