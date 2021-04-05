use gtk::{
    AdjustmentExt, ContainerExt, Label, LabelExt, Orientation, PositionType, RangeExt, Scale,
    ScaleExt, Switch, SwitchExt, WidgetExt,
};
use gtk::{
    BoxExt,
    Orientation::{Horizontal, Vertical},
};

use relm::{connect, Relm};

use wvr_data::{DataHolder, DataRange};

use super::{RenderStageConfigView, RenderStageConfigViewMsg};

fn create_int_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    value: i64,
    value_range: &DataRange,
) -> gtk::Box {
    let (min_value, max_value, step) =
        if let DataRange::IntRange(min_value, max_value, step) = value_range {
            (*min_value as f64, *max_value as f64, *step as f64)
        } else {
            (-8192.0, 8192.0, 1.0)
        };

    let spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    spinner.set_value(value as f64);
    spinner.set_hexpand(true);

    spinner.set_value_pos(PositionType::Right);

    let name = name.to_string();
    connect!(relm, spinner, connect_value_changed(val), {
        Some(RenderStageConfigViewMsg::UpdateVariable(
            name.clone(),
            DataHolder::Int(val.get_value() as i32),
        ))
    });

    let wrapper = gtk::Box::new(Horizontal, 0);
    wrapper.pack_end(&spinner, true, true, 0);

    wrapper
}
fn create_float_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    value: f64,
    value_range: &DataRange,
) -> gtk::Box {
    let (min_value, max_value, step) =
        if let DataRange::FloatRange(min_value, max_value, step) = value_range {
            (*min_value, *max_value, *step)
        } else {
            (-1.0, 1.0, 0.0001)
        };

    let spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    spinner.set_value(value as f64);
    spinner.set_hexpand(true);

    spinner.set_value_pos(PositionType::Right);

    let name = name.to_string();
    connect!(relm, spinner, connect_value_changed(val), {
        Some(RenderStageConfigViewMsg::UpdateVariable(
            name.clone(),
            DataHolder::Float(val.get_value() as f32),
        ))
    });

    let wrapper = gtk::Box::new(Horizontal, 0);
    wrapper.pack_end(&spinner, true, true, 0);

    wrapper
}

fn create_float2_spinner(
    relm: &Relm<RenderStageConfigView>,
    name: &str,
    x: f64,
    y: f64,
    value_range: &DataRange,
) -> gtk::Box {
    let (min_value, max_value, step) =
        if let DataRange::FloatRange(min_value, max_value, step) = value_range {
            (*min_value, *max_value, *step)
        } else {
            (-1.0, 1.0, 0.0001)
        };

    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    x_spinner.set_value(x as f64);
    x_spinner.set_hexpand(true);

    x_spinner.set_value_pos(PositionType::Right);

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);

    let y_label = Label::new(Some("y"));
    let y_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    y_spinner.set_value(y as f64);
    y_spinner.set_hexpand(true);

    y_spinner.set_value_pos(PositionType::Right);

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    components_wrapper.add(&x_wrapper);
    components_wrapper.add(&y_wrapper);

    let name = name.to_string();
    {
        let name = name.clone();
        let x_spinner = x_spinner.clone();
        let y_spinner = y_spinner.clone();
        connect!(relm, x_spinner, connect_value_changed(val), {
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
        connect!(relm, y_spinner, connect_value_changed(val), {
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
    value_range: &DataRange,
) -> gtk::Box {
    let (min_value, max_value, step) =
        if let DataRange::FloatRange(min_value, max_value, step) = value_range {
            (*min_value, *max_value, *step)
        } else {
            (-1.0, 1.0, 0.0001)
        };

    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    x_spinner.set_value(x as f64);
    x_spinner.set_hexpand(true);

    x_spinner.set_value_pos(PositionType::Right);

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);

    let y_label = Label::new(Some("y"));
    let y_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    y_spinner.set_value(y as f64);
    y_spinner.set_hexpand(true);

    y_spinner.set_value_pos(PositionType::Right);

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    let z_wrapper = gtk::Box::new(Horizontal, 2);

    let z_label = Label::new(Some("z"));
    let z_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    z_spinner.set_value(z as f64);
    z_spinner.set_hexpand(true);

    z_spinner.set_value_pos(PositionType::Right);

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
        connect!(relm, x_spinner, connect_value_changed(val), {
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
        connect!(relm, y_spinner, connect_value_changed(val), {
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
        connect!(relm, z_spinner, connect_value_changed(val), {
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
    value_range: &DataRange,
) -> gtk::Box {
    let (min_value, max_value, step) =
        if let DataRange::FloatRange(min_value, max_value, step) = value_range {
            (*min_value, *max_value, *step)
        } else {
            (-1.0, 1.0, 0.0001)
        };

    let components_wrapper = gtk::Box::new(Vertical, 2);

    let x_wrapper = gtk::Box::new(Horizontal, 2);

    let x_label = Label::new(Some("x"));
    x_label.set_text("x");
    let x_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    x_spinner.set_value(x as f64);
    x_spinner.set_hexpand(true);

    x_spinner.set_value_pos(PositionType::Right);

    x_wrapper.add(&x_label);
    x_wrapper.add(&x_spinner);

    let y_wrapper = gtk::Box::new(Horizontal, 2);

    let y_label = Label::new(Some("y"));
    let y_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    y_spinner.set_value(y as f64);
    y_spinner.set_hexpand(true);

    y_spinner.set_value_pos(PositionType::Right);

    y_wrapper.add(&y_label);
    y_wrapper.add(&y_spinner);

    let z_wrapper = gtk::Box::new(Horizontal, 2);

    let z_label = Label::new(Some("z"));
    let z_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    z_spinner.set_value(z as f64);
    z_spinner.set_hexpand(true);

    z_spinner.set_value_pos(PositionType::Right);

    z_wrapper.add(&z_label);
    z_wrapper.add(&z_spinner);

    let w_wrapper = gtk::Box::new(Horizontal, 2);

    let w_label = Label::new(Some("w"));
    let w_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    w_spinner.set_value(w as f64);
    w_spinner.set_hexpand(true);

    w_spinner.set_value_pos(PositionType::Right);

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
        connect!(relm, x_spinner, connect_value_changed(val), {
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
        connect!(relm, y_spinner, connect_value_changed(val), {
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
        connect!(relm, z_spinner, connect_value_changed(val), {
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
        connect!(relm, w_spinner, connect_value_changed(val), {
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
    variable_range: &DataRange,
) -> gtk::Box {
    return match variable_value {
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

            let wrapper = gtk::Box::new(Horizontal, 0);
            wrapper.pack_end(&variable_switch, false, false, 0);

            wrapper
        }
        DataHolder::Int(value) => {
            create_int_spinner(relm, variable_name, *value as i64, variable_range)
        }
        DataHolder::Float(value) => {
            create_float_spinner(relm, variable_name, *value as f64, variable_range)
        }
        DataHolder::Float2(value) => create_float2_spinner(
            relm,
            variable_name,
            value[0] as f64,
            value[1] as f64,
            variable_range,
        ),
        DataHolder::Float3(value) => create_float3_spinner(
            relm,
            variable_name,
            value[0] as f64,
            value[1] as f64,
            value[2] as f64,
            variable_range,
        ),
        DataHolder::Float4(value) => create_float4_spinner(
            relm,
            variable_name,
            value[0] as f64,
            value[1] as f64,
            value[2] as f64,
            value[3] as f64,
            variable_range,
        ),
        _ => unimplemented!(),
    };
}
