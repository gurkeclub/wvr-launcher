use gdk::RGBA;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, Align, ContainerExt, EditableSignals, Label, LabelExt, SpinButton, StateFlags,
    Switch, SwitchExt, WidgetExt,
};

use relm::{connect, Relm};

use wvr_data::DataHolder;

use super::{RenderStageConfigView, RenderStageConfigViewMsg};

pub enum VariableInput {
    Bool(Switch),
    Int(SpinButton),
    Float(SpinButton),
    Vec2(SpinButton, SpinButton),
    Vec3(SpinButton, SpinButton, SpinButton),
    Vec4(SpinButton, SpinButton, SpinButton, SpinButton),
}

fn create_int_spinner(relm: &Relm<RenderStageConfigView>, value: i64) -> SpinButton {
    let spinner = SpinButton::new(
        Some(&Adjustment::new(
            value as f64,
            -8192.0,
            8192.0,
            1.0,
            10.0,
            10.0,
        )),
        1.0,
        0,
    );

    connect!(
        relm,
        spinner,
        connect_changed(_),
        Some(RenderStageConfigViewMsg::UpdateVariableList)
    );

    spinner
}
fn create_float_spinner(relm: &Relm<RenderStageConfigView>, value: f64) -> SpinButton {
    let spinner = SpinButton::new(
        Some(&Adjustment::new(value, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    connect!(
        relm,
        spinner,
        connect_changed(_),
        Some(RenderStageConfigViewMsg::UpdateVariableList)
    );

    spinner
}

pub fn build_variable_row(
    relm: &Relm<RenderStageConfigView>,
    variable_name: &str,
    variable_value: &DataHolder,
) -> (gtk::Box, VariableInput) {
    let outer_wrapper = gtk::Box::new(Horizontal, 2);
    outer_wrapper.override_background_color(
        StateFlags::NORMAL,
        Some(&RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0625,
        }),
    );

    let wrapper = gtk::Box::new(Horizontal, 2);
    wrapper.set_property_margin(4);

    let variable_name_label = Label::new(Some(variable_name));
    variable_name_label.set_xalign(0.0);
    variable_name_label.set_size_request(48, 0);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    wrapper.add(&variable_name_label);
    wrapper.add(&padding);

    let variable_input = match variable_value {
        DataHolder::Bool(value) => {
            let variable_switch = Switch::new();
            variable_switch.set_state(*value);
            connect!(
                relm,
                variable_switch,
                connect_property_active_notify(_),
                Some(RenderStageConfigViewMsg::UpdateVariableList)
            );
            wrapper.add(&variable_switch);
            VariableInput::Bool(variable_switch)
        }
        DataHolder::Int(value) => {
            let variable_spinner = create_int_spinner(relm, *value as i64);

            wrapper.add(&variable_spinner);
            VariableInput::Int(variable_spinner)
        }
        DataHolder::Float(value) => {
            let variable_spinner = create_float_spinner(relm, *value as f64);

            wrapper.add(&variable_spinner);
            VariableInput::Float(variable_spinner)
        }
        DataHolder::Float2(value) => {
            let components_wrapper = gtk::Box::new(Vertical, 2);

            let x_wrapper = gtk::Box::new(Horizontal, 2);
            x_wrapper.set_halign(Align::End);

            let x_label = Label::new(Some("x"));
            x_label.set_text("x");
            let x_spinner = create_float_spinner(relm, value[0] as f64);

            x_wrapper.add(&x_label);
            x_wrapper.add(&x_spinner);

            let y_wrapper = gtk::Box::new(Horizontal, 2);
            y_wrapper.set_halign(Align::End);

            let y_label = Label::new(Some("y"));
            let y_spinner = create_float_spinner(relm, value[1] as f64);

            y_wrapper.add(&y_label);
            y_wrapper.add(&y_spinner);

            components_wrapper.add(&x_wrapper);
            components_wrapper.add(&y_wrapper);

            wrapper.add(&components_wrapper);

            VariableInput::Vec2(x_spinner, y_spinner)
        }
        DataHolder::Float3(value) => {
            let components_wrapper = gtk::Box::new(Vertical, 2);

            let x_wrapper = gtk::Box::new(Horizontal, 2);
            x_wrapper.set_halign(Align::End);

            let x_label = Label::new(Some("x"));
            x_label.set_text("x");
            let x_spinner = create_float_spinner(relm, value[0] as f64);

            x_wrapper.add(&x_label);
            x_wrapper.add(&x_spinner);

            let y_wrapper = gtk::Box::new(Horizontal, 2);
            y_wrapper.set_halign(Align::End);

            let y_label = Label::new(Some("y"));
            let y_spinner = create_float_spinner(relm, value[1] as f64);

            y_wrapper.add(&y_label);
            y_wrapper.add(&y_spinner);

            let z_wrapper = gtk::Box::new(Horizontal, 2);
            z_wrapper.set_halign(Align::End);

            let z_label = Label::new(Some("z"));
            let z_spinner = create_float_spinner(relm, value[2] as f64);

            z_wrapper.add(&z_label);
            z_wrapper.add(&z_spinner);

            components_wrapper.add(&x_wrapper);
            components_wrapper.add(&y_wrapper);
            components_wrapper.add(&z_wrapper);

            wrapper.add(&components_wrapper);

            VariableInput::Vec3(x_spinner, y_spinner, z_spinner)
        }
        DataHolder::Float4(value) => {
            let components_wrapper = gtk::Box::new(Vertical, 2);

            let x_wrapper = gtk::Box::new(Horizontal, 2);
            x_wrapper.set_halign(Align::End);

            let x_label = Label::new(Some("x"));
            x_label.set_text("x");
            let x_spinner = create_float_spinner(relm, value[0] as f64);

            x_wrapper.add(&x_label);
            x_wrapper.add(&x_spinner);

            let y_wrapper = gtk::Box::new(Horizontal, 2);
            y_wrapper.set_halign(Align::End);

            let y_label = Label::new(Some("y"));
            let y_spinner = create_float_spinner(relm, value[1] as f64);

            y_wrapper.add(&y_label);
            y_wrapper.add(&y_spinner);

            let z_wrapper = gtk::Box::new(Horizontal, 2);
            z_wrapper.set_halign(Align::End);

            let z_label = Label::new(Some("z"));
            let z_spinner = create_float_spinner(relm, value[2] as f64);

            z_wrapper.add(&z_label);
            z_wrapper.add(&z_spinner);

            let w_wrapper = gtk::Box::new(Horizontal, 2);
            w_wrapper.set_halign(Align::End);

            let w_label = Label::new(Some("z"));
            let w_spinner = create_float_spinner(relm, value[3] as f64);

            w_wrapper.add(&w_label);
            w_wrapper.add(&w_spinner);

            components_wrapper.add(&x_wrapper);
            components_wrapper.add(&y_wrapper);
            components_wrapper.add(&z_wrapper);
            components_wrapper.add(&w_wrapper);

            wrapper.add(&components_wrapper);

            VariableInput::Vec4(x_spinner, y_spinner, z_spinner, w_spinner)
        }
        _ => unimplemented!(),
    };

    outer_wrapper.add(&wrapper);

    (outer_wrapper, variable_input)
}
