use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, AdjustmentExt, Align, ContainerExt, EditableSignals, Label, LabelExt, SpinButton,
    SpinButtonExt, SpinButtonSignals, Switch, SwitchExt, WidgetExt,
};

use relm::{connect, Relm};

use wvr_data::DataHolder;

use super::{RenderStageConfigView, RenderStageConfigViewMsg};

fn create_int_spinner(relm: &Relm<RenderStageConfigView>, name: &str, value: i64) -> SpinButton {
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

    let name = name.to_string();
    connect!(relm, spinner, connect_changed(val), {
        val.update();
        Some(RenderStageConfigViewMsg::UpdateVariable(
            name.clone(),
            DataHolder::Int(val.get_value() as i32),
        ))
    });

    spinner
}
fn create_float_spinner(relm: &Relm<RenderStageConfigView>, name: &str, value: f64) -> SpinButton {
    let spinner = SpinButton::new(
        Some(&Adjustment::new(value, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    let name = name.to_string();
    connect!(relm, spinner, connect_value_changed(val), {
        val.update();
        Some(RenderStageConfigViewMsg::UpdateVariable(
            name.clone(),
            DataHolder::Float(val.get_value() as f32),
        ))
    });

    spinner
}

fn create_float2_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    x: f64,
    y: f64,
) -> gtk::Box {
    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);
    x_wrapper.set_halign(Align::End);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = SpinButton::new(
        Some(&Adjustment::new(x, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);
    y_wrapper.set_halign(Align::End);

    let y_label = Label::new(Some("y"));
    let y_spinner = SpinButton::new(
        Some(&Adjustment::new(y, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    components_wrapper.add(&x_wrapper);
    components_wrapper.add(&y_wrapper);

    let name = name.to_string();
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        connect!(relm, x_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float2([val.get_value() as f32, y_spinner.get_value() as f32]),
            ))
        });
    }

    {
        let name = name;
        let x_spinner = x_spinner;
        let y_spinner = y_spinner;
        connect!(relm, y_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float2([x_spinner.get_value() as f32, val.get_value() as f32]),
            ))
        });
    }

    components_wrapper
}

fn create_float3_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
) -> gtk::Box {
    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);
    x_wrapper.set_halign(Align::End);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = SpinButton::new(
        Some(&Adjustment::new(x, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);
    y_wrapper.set_halign(Align::End);

    let y_label = Label::new(Some("y"));
    let y_spinner = SpinButton::new(
        Some(&Adjustment::new(y, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    let z_wrapper = gtk::Box::new(Horizontal, 2);
    z_wrapper.set_halign(Align::End);

    let z_label = Label::new(Some("z"));
    let z_spinner = SpinButton::new(
        Some(&Adjustment::new(z, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    z_wrapper.add(&z_label);
    z_wrapper.add(&z_spinner);

    components_wrapper.add(&x_wrapper);
    components_wrapper.add(&y_wrapper);
    components_wrapper.add(&z_wrapper);

    let name = name.to_string();
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        let z_spinner = z_spinner.clone();
        connect!(relm, x_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float3([
                    val.get_adjustment().get_value() as f32,
                    y_spinner.get_adjustment().get_value() as f32,
                    z_spinner.get_adjustment().get_value() as f32,
                ]),
            ))
        });
    }
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        let z_spinner = z_spinner.clone();
        connect!(relm, y_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float3([
                    x_spinner.get_value() as f32,
                    val.get_value() as f32,
                    z_spinner.get_value() as f32,
                ]),
            ))
        });
    }
    {
        let name = name;
        let x_spinner = x_spinner;
        let y_spinner = y_spinner;
        let z_spinner = z_spinner;
        connect!(relm, z_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float3([
                    x_spinner.get_value() as f32,
                    y_spinner.get_value() as f32,
                    val.get_value() as f32,
                ]),
            ))
        });
    }

    components_wrapper
}

fn create_float4_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) -> gtk::Box {
    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);
    x_wrapper.set_halign(Align::End);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = SpinButton::new(
        Some(&Adjustment::new(x, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);
    y_wrapper.set_halign(Align::End);

    let y_label = Label::new(Some("y"));
    let y_spinner = SpinButton::new(
        Some(&Adjustment::new(y, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    let z_wrapper = gtk::Box::new(Horizontal, 2);
    z_wrapper.set_halign(Align::End);

    let z_label = Label::new(Some("z"));
    let z_spinner = SpinButton::new(
        Some(&Adjustment::new(z, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    z_wrapper.add(&z_label);
    z_wrapper.add(&z_spinner);

    let w_wrapper = gtk::Box::new(Horizontal, 2);
    w_wrapper.set_halign(Align::End);

    let w_label = Label::new(Some("w"));
    let w_spinner = SpinButton::new(
        Some(&Adjustment::new(w, -8192.0, 8192.0, 0.001, 0.01, 0.01)),
        0.01,
        6,
    );

    w_wrapper.add(&w_label);
    w_wrapper.add(&w_spinner);

    components_wrapper.add(&x_wrapper);
    components_wrapper.add(&y_wrapper);
    components_wrapper.add(&z_wrapper);
    components_wrapper.add(&w_wrapper);

    let name = name.to_string();
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        let z_spinner = z_spinner.clone();
        let w_spinner = w_spinner.clone();
        connect!(relm, x_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float4([
                    val.get_value() as f32,
                    y_spinner.get_value() as f32,
                    z_spinner.get_value() as f32,
                    w_spinner.get_value() as f32,
                ]),
            ))
        });
    }
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        let z_spinner = z_spinner.clone();
        let w_spinner = w_spinner.clone();
        connect!(relm, y_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float4([
                    x_spinner.get_value() as f32,
                    val.get_value() as f32,
                    z_spinner.get_value() as f32,
                    w_spinner.get_value() as f32,
                ]),
            ))
        });
    }

    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        let z_spinner = z_spinner.clone();
        let w_spinner = w_spinner.clone();
        connect!(relm, z_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float4([
                    x_spinner.get_value() as f32,
                    y_spinner.get_value() as f32,
                    val.get_value() as f32,
                    w_spinner.get_value() as f32,
                ]),
            ))
        });
    }

    {
        let name = name;
        let x_spinner = x_spinner;
        let y_spinner = y_spinner;
        let z_spinner = z_spinner;
        let w_spinner = w_spinner;
        connect!(relm, w_spinner, connect_changed(val), {
            val.update();
            Some(RenderStageConfigViewMsg::UpdateVariable(
                name.clone(),
                DataHolder::Float4([
                    x_spinner.get_value() as f32,
                    y_spinner.get_value() as f32,
                    z_spinner.get_value() as f32,
                    val.get_value() as f32,
                ]),
            ))
        });
    }

    components_wrapper
}

pub fn build_variable_row(
    relm: &Relm<RenderStageConfigView>,
    variable_name: &str,
    variable_value: &DataHolder,
) -> gtk::Box {
    let outer_wrapper = gtk::Box::new(Horizontal, 2);

    let wrapper = gtk::Box::new(Horizontal, 2);
    wrapper.set_property_margin(4);

    let variable_name_label = Label::new(Some(variable_name));
    variable_name_label.set_xalign(0.0);
    variable_name_label.set_size_request(48, 0);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    wrapper.add(&variable_name_label);
    wrapper.add(&padding);

    match variable_value {
        DataHolder::Bool(value) => {
            let variable_switch = Switch::new();
            variable_switch.set_state(*value);

            let variable_name = variable_name.to_string();
            connect!(
                relm,
                variable_switch,
                connect_property_active_notify(val),
                Some(RenderStageConfigViewMsg::UpdateVariable(
                    variable_name.clone(),
                    DataHolder::Bool(val.get_active())
                ))
            );
            wrapper.add(&variable_switch);
        }
        DataHolder::Int(value) => {
            let variable_spinner = create_int_spinner(relm, variable_name, *value as i64);

            wrapper.add(&variable_spinner);
        }
        DataHolder::Float(value) => {
            let variable_spinner = create_float_spinner(relm, variable_name, *value as f64);

            wrapper.add(&variable_spinner);
        }
        DataHolder::Float2(value) => {
            let components_wrapper =
                create_float2_spinner(relm, variable_name, value[0] as f64, value[1] as f64);

            wrapper.add(&components_wrapper);
        }
        DataHolder::Float3(value) => {
            let components_wrapper = create_float3_spinner(
                relm,
                variable_name,
                value[0] as f64,
                value[1] as f64,
                value[2] as f64,
            );

            wrapper.add(&components_wrapper);
        }
        DataHolder::Float4(value) => {
            let components_wrapper = create_float4_spinner(
                relm,
                variable_name,
                value[0] as f64,
                value[1] as f64,
                value[2] as f64,
                value[3] as f64,
            );

            wrapper.add(&components_wrapper);
        }
        _ => unimplemented!(),
    }

    outer_wrapper.add(&wrapper);

    outer_wrapper
}
