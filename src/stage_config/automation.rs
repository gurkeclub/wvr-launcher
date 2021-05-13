use gtk::{
    prelude::{GtkListStoreExtManual, TreeSortableExtManual},
    Button, ButtonExt, ComboBoxExt, ComboBoxText, ContainerExt, Grid, GridExt, Label, LabelExt,
    MenuButton, MenuButtonExt, OrientableExt, Orientation, Popover, RangeExt, Scale, ScaleExt,
    SortColumn, SortType, Switch, SwitchExt, WidgetExt,
};
use gtk::{
    BoxExt,
    Orientation::{Horizontal, Vertical},
};

use relm::{connect, Relm};

use wvr_data::config::project_config::{Automation, LfoType};
use wvr_data::DataRange;

use super::{
    list_store_sort_function,
    view::{RenderStageConfigView, RenderStageConfigViewMsg},
};

fn build_lfo_row(
    relm: &Relm<RenderStageConfigView>,
    variable_name: &str,
    variable_range: &DataRange,
    lfo_type: LfoType,
    numerator: f64,
    denominator: f64,
    phase: f64,
    amplitude: f64,
    signed: bool,
) -> Grid {
    let automation_container = Grid::new();
    automation_container.set_property_margin(4);
    automation_container.set_row_spacing(4);
    automation_container.set_column_spacing(4);
    automation_container.set_orientation(Orientation::Vertical);

    let lfo_type_label = Label::new(Some("Type: "));
    lfo_type_label.set_xalign(0.0);
    automation_container.attach(&lfo_type_label, 0, 0, 1, 1);

    let available_lfo_types = ["Triangle", "Saw", "Sine", "Square"];
    let lfo_type_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
    for name in available_lfo_types.iter() {
        lfo_type_store.insert_with_values(None, &[0, 1], &[name, name]);
    }
    lfo_type_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
    lfo_type_store.set_default_sort_func(&list_store_sort_function);

    let lfo_type_chooser = gtk::ComboBoxText::new();
    lfo_type_chooser.set_hexpand(false);
    lfo_type_chooser.set_model(Some(&lfo_type_store));
    lfo_type_chooser.set_tooltip_text(Some("Generated buffer precision"));

    lfo_type_chooser.set_id_column(0);
    lfo_type_chooser.set_entry_text_column(1);
    automation_container.attach(&lfo_type_chooser, 1, 0, 1, 1);

    match lfo_type {
        LfoType::Triangle => lfo_type_chooser.set_active_id(Some("Triangle")),
        LfoType::Saw => lfo_type_chooser.set_active_id(Some("Saw")),
        LfoType::Sine => lfo_type_chooser.set_active_id(Some("Triangle")),
        LfoType::Square => lfo_type_chooser.set_active_id(Some("Square")),
    };

    let numerator_label = Label::new(Some("Numerator: "));
    numerator_label.set_xalign(0.0);
    automation_container.attach(&numerator_label, 0, 1, 1, 1);

    let numerator_spinner = Scale::with_range(Orientation::Horizontal, 1.0, 32.0, 1.0);
    numerator_spinner.set_value(numerator);
    numerator_spinner.set_hexpand(true);

    automation_container.attach(&numerator_spinner, 1, 1, 1, 1);

    let denominator_label = Label::new(Some("Denominator: "));
    denominator_label.set_xalign(0.0);
    automation_container.attach(&denominator_label, 0, 2, 1, 1);

    let denominator_spinner = Scale::with_range(Orientation::Horizontal, 1.0, 32.0, 1.0);
    denominator_spinner.set_value(denominator);
    denominator_spinner.set_hexpand(true);
    automation_container.attach(&denominator_spinner, 1, 2, 1, 1);

    let phase_label = Label::new(Some("Phase: "));
    phase_label.set_xalign(0.0);
    automation_container.attach(&phase_label, 0, 3, 1, 1);

    let phase_spinner = Scale::with_range(Orientation::Horizontal, -1.0, 1.0, 0.001);
    phase_spinner.set_has_origin(false);
    phase_spinner.set_value(phase);
    phase_spinner.set_hexpand(true);
    automation_container.attach(&phase_spinner, 1, 3, 1, 1);

    let amplitude_label = Label::new(Some("Amplitude: "));
    amplitude_label.set_xalign(0.0);
    automation_container.attach(&amplitude_label, 0, 4, 1, 1);

    let (min_value, max_value, step) = match *variable_range {
        DataRange::FloatRange(min_value, max_value, step) => (
            -(max_value - min_value).abs() as f64,
            (max_value - min_value).abs(),
            step,
        ),
        DataRange::IntRange(min_value, max_value, step) => (
            -(max_value - min_value).abs() as f64,
            (max_value - min_value).abs() as f64,
            step as f64,
        ),
        _ => (0.0, 1.0, 1.0),
    };

    let amplitude_spinner = Scale::with_range(Orientation::Horizontal, min_value, max_value, step);
    amplitude_spinner.set_has_origin(false);
    amplitude_spinner.set_value(amplitude);
    amplitude_spinner.set_hexpand(true);

    automation_container.attach(&amplitude_spinner, 1, 4, 1, 1);

    let signed_label = Label::new(Some("Signed: "));
    signed_label.set_xalign(0.0);
    automation_container.attach(&signed_label, 0, 5, 1, 1);

    let signed_toggler = Switch::new();
    signed_toggler.set_state(signed);

    let signed_toggler_wrapper = gtk::Box::new(Horizontal, 0);
    signed_toggler_wrapper.set_hexpand(true);
    signed_toggler_wrapper.pack_end(&signed_toggler, false, false, 0);

    automation_container.attach(&signed_toggler_wrapper, 1, 5, 1, 1);

    let update_automation = |name: &str,
                             lfo_type_chooser: &ComboBoxText,
                             numerator_spinner: &Scale,
                             denominator_spinner: &Scale,
                             phase_spinner: &Scale,
                             amplitude_spinner: &Scale,
                             signed_toggler_wrapper: &Switch| {
        let lfo_type = match lfo_type_chooser.get_active_id().unwrap().as_str() {
            "Sine" => LfoType::Sine,
            "Square" => LfoType::Square,
            "Saw" => LfoType::Saw,
            "Triangle" => LfoType::Triangle,
            _ => unreachable!(),
        };
        Some(RenderStageConfigViewMsg::UpdateVariableAutomation(
            name.to_string(),
            Automation::Lfo(
                lfo_type,
                numerator_spinner.get_value().max(1.0),
                denominator_spinner.get_value().max(1.0),
                phase_spinner.get_value(),
                amplitude_spinner.get_value(),
                signed_toggler_wrapper.get_active(),
            ),
        ))
    };

    let variable_name = variable_name.to_string();
    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(relm, numerator_spinner.clone(), connect_value_changed(_), {
            update_automation(
                &variable_name,
                &lfo_type_chooser,
                &numerator_spinner,
                &denominator_spinner,
                &phase_spinner,
                &amplitude_spinner,
                &signed_toggler,
            )
        });
    }

    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(
            relm,
            denominator_spinner.clone(),
            connect_value_changed(_),
            {
                update_automation(
                    &variable_name,
                    &lfo_type_chooser,
                    &numerator_spinner,
                    &denominator_spinner,
                    &phase_spinner,
                    &amplitude_spinner,
                    &signed_toggler,
                )
            }
        );
    }

    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(relm, phase_spinner.clone(), connect_value_changed(_), {
            update_automation(
                &variable_name,
                &lfo_type_chooser,
                &numerator_spinner,
                &denominator_spinner,
                &phase_spinner,
                &amplitude_spinner,
                &signed_toggler,
            )
        });
    }

    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(relm, amplitude_spinner.clone(), connect_value_changed(_), {
            update_automation(
                &variable_name,
                &lfo_type_chooser,
                &numerator_spinner,
                &denominator_spinner,
                &phase_spinner,
                &amplitude_spinner,
                &signed_toggler,
            )
        });
    }

    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(
            relm,
            signed_toggler.clone(),
            connect_property_active_notify(_),
            {
                update_automation(
                    &variable_name,
                    &lfo_type_chooser,
                    &numerator_spinner,
                    &denominator_spinner,
                    &phase_spinner,
                    &amplitude_spinner,
                    &signed_toggler,
                )
            }
        );
    }

    {
        let variable_name = variable_name.clone();
        let lfo_type_chooser = lfo_type_chooser.clone();
        let numerator_spinner = numerator_spinner.clone();
        let denominator_spinner = denominator_spinner.clone();
        let phase_spinner = phase_spinner.clone();
        let amplitude_spinner = amplitude_spinner.clone();
        let signed_toggler = signed_toggler.clone();

        connect!(relm, lfo_type_chooser.clone(), connect_changed(_), {
            update_automation(
                &variable_name,
                &lfo_type_chooser,
                &numerator_spinner,
                &denominator_spinner,
                &phase_spinner,
                &amplitude_spinner,
                &signed_toggler,
            )
        });
    }

    automation_container
}

fn build_add_lfo_button(
    relm: &Relm<RenderStageConfigView>,
    variable_name: &str,
    variable_range: &DataRange,
    automation_wrapper: &gtk::Box,
    automation_button_label: &Label,
) -> Button {
    let relm = relm.clone();
    let variable_name = variable_name.to_owned();
    let variable_range = variable_range.to_owned();
    let automation_wrapper = automation_wrapper.clone();
    let automation_button_label = automation_button_label.clone();

    let add_lfo_button = Button::new();
    add_lfo_button.set_label("Add LFO");
    add_lfo_button.set_property_margin(4);

    connect!(relm, add_lfo_button, connect_clicked(add_lfo_button), {
        for child in automation_wrapper.get_children().iter().skip(1) {
            automation_wrapper.remove(child);
        }
        automation_wrapper.add(&build_lfo_row(
            &relm,
            &variable_name,
            &variable_range,
            LfoType::Sine,
            1.0,
            8.0,
            0.0,
            0.0,
            false,
        ));
        automation_button_label.set_text("LFO");
        add_lfo_button.set_label("Replace LFO");

        automation_wrapper.show_all();
    });

    add_lfo_button
}

pub fn build_automation_row(
    relm: &Relm<RenderStageConfigView>,
    variable_name: &str,
    variable_automation: &Automation,
    variable_range: &DataRange,
) -> MenuButton {
    let button_label = match variable_automation {
        Automation::None => emoji::objects::tool::GEAR,
        Automation::Lfo(_, _, _, _, _, _) => "LFO",
    };

    let automation_button_label = Label::new(Some(button_label));
    let automation_button = MenuButton::new();
    automation_button.add(&automation_button_label);

    let automation_popover = Popover::new(Some(&automation_button));
    automation_button.set_popover(Some(&automation_popover));

    let automation_wrapper = gtk::Box::new(Vertical, 4);
    automation_wrapper.set_property_width_request(320);

    automation_wrapper.add(&build_add_lfo_button(
        relm,
        variable_name,
        variable_range,
        &automation_wrapper,
        &automation_button_label,
    ));

    match *variable_automation {
        Automation::Lfo(lfo_type, numerator, denominator, phase, amplitude, signed) => {
            automation_wrapper.add(&build_lfo_row(
                relm,
                variable_name,
                variable_range,
                lfo_type,
                numerator,
                denominator,
                phase,
                amplitude,
                signed,
            ));
        }
        Automation::None => (),
    }

    automation_popover.add(&automation_wrapper);
    automation_wrapper.show_all();

    automation_button
}
