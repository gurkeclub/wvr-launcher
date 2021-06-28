use gtk::{
    prelude::{GtkListStoreExtManual, TreeSortableExtManual},
    Button, ButtonExt, ComboBoxExt, ContainerExt, Grid, GridExt, Label, LabelExt, MenuButton,
    MenuButtonExt, OrientableExt, Orientation, Popover, RangeExt, Scale, ScaleExt, SortColumn,
    SortType, Switch, SwitchExt, WidgetExt,
};
use gtk::{
    BoxExt,
    Orientation::{Horizontal, Vertical},
};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;

use wvr_data::config::project_config::{Automation, Lfo, LfoType};
use wvr_data::DataRange;

use super::{
    list_store_sort_function,
    view::{RenderStageConfigView, RenderStageConfigViewMsg},
};

pub fn build_automation_selector(
    parent_relm: Relm<RenderStageConfigView>,
    variable_name: String,
    variable_dimension_count: usize,
    variable_range: DataRange,
    automation_config: Automation,
) -> (Component<AutomationView>, gtk::Box) {
    let automation_button_wrapper = gtk::Box::new(Horizontal, 0);
    let automation_button = automation_button_wrapper.add_widget::<AutomationView>((
        parent_relm,
        variable_name,
        variable_dimension_count,
        variable_range,
        automation_config,
    ));

    (automation_button, automation_button_wrapper)
}

#[derive(Msg)]
pub enum AutomationViewMsg {
    SetType(usize, String),
    SetNumerator(usize, f64),
    SetDenominator(usize, f64),
    SetPhase(usize, f64),
    SetAmplitude(usize, f64),
    SetSigned(usize, bool),
    SetAutomation(Automation),
}

impl AutomationViewMsg {
    pub fn get_target_index(&self) -> Option<usize> {
        match self {
            AutomationViewMsg::SetAmplitude(index, _) => Some(*index),
            AutomationViewMsg::SetDenominator(index, _) => Some(*index),
            AutomationViewMsg::SetNumerator(index, _) => Some(*index),
            AutomationViewMsg::SetPhase(index, _) => Some(*index),
            AutomationViewMsg::SetSigned(index, _) => Some(*index),
            AutomationViewMsg::SetType(index, _) => Some(*index),
            _ => None,
        }
    }

    pub fn update_lfo(&self, lfo: &mut Lfo) {
        match self {
            AutomationViewMsg::SetAmplitude(_, amplitude) => {
                lfo.amplitude = *amplitude;
            }
            AutomationViewMsg::SetDenominator(_, denominator) => {
                lfo.denominator = *denominator;
            }
            AutomationViewMsg::SetNumerator(_, numerator) => {
                lfo.numerator = *numerator;
            }
            AutomationViewMsg::SetPhase(_, phase) => {
                lfo.phase = *phase;
            }
            AutomationViewMsg::SetSigned(_, signed) => {
                lfo.signed = *signed;
            }
            AutomationViewMsg::SetType(_, lfo_type) => match lfo_type.as_str() {
                "Saw" => lfo.lfo_type = LfoType::Saw,
                "Sine" => lfo.lfo_type = LfoType::Sine,
                "Square" => lfo.lfo_type = LfoType::Square,
                "Triangle" => lfo.lfo_type = LfoType::Triangle,
                _ => (),
            },
            _ => (),
        }
    }
}

fn build_lfo_row(
    relm: &Relm<AutomationView>,
    variable_range: &DataRange,
    target_dimension: usize,
    lfo: Lfo,
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

    match lfo.lfo_type {
        LfoType::Triangle => lfo_type_chooser.set_active_id(Some("Triangle")),
        LfoType::Saw => lfo_type_chooser.set_active_id(Some("Saw")),
        LfoType::Sine => lfo_type_chooser.set_active_id(Some("Triangle")),
        LfoType::Square => lfo_type_chooser.set_active_id(Some("Square")),
    };

    let numerator_label = Label::new(Some("Numerator: "));
    numerator_label.set_xalign(0.0);
    automation_container.attach(&numerator_label, 0, 1, 1, 1);

    let numerator_spinner = Scale::with_range(Orientation::Horizontal, 1.0, 32.0, 1.0);
    numerator_spinner.set_value(lfo.numerator);
    numerator_spinner.set_hexpand(true);

    automation_container.attach(&numerator_spinner, 1, 1, 1, 1);

    let denominator_label = Label::new(Some("Denominator: "));
    denominator_label.set_xalign(0.0);
    automation_container.attach(&denominator_label, 0, 2, 1, 1);

    let denominator_spinner = Scale::with_range(Orientation::Horizontal, 1.0, 32.0, 1.0);
    denominator_spinner.set_value(lfo.denominator);
    denominator_spinner.set_hexpand(true);
    automation_container.attach(&denominator_spinner, 1, 2, 1, 1);

    let phase_label = Label::new(Some("Phase: "));
    phase_label.set_xalign(0.0);
    automation_container.attach(&phase_label, 0, 3, 1, 1);

    let phase_spinner = Scale::with_range(Orientation::Horizontal, -1.0, 1.0, 0.001);
    phase_spinner.set_has_origin(false);
    phase_spinner.set_value(lfo.phase);
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
    amplitude_spinner.set_value(lfo.amplitude);
    amplitude_spinner.set_hexpand(true);

    automation_container.attach(&amplitude_spinner, 1, 4, 1, 1);

    let signed_label = Label::new(Some("Signed: "));
    signed_label.set_xalign(0.0);
    automation_container.attach(&signed_label, 0, 5, 1, 1);

    let signed_toggler = Switch::new();
    signed_toggler.set_state(lfo.signed);

    let signed_toggler_wrapper = gtk::Box::new(Horizontal, 0);
    signed_toggler_wrapper.set_hexpand(true);
    signed_toggler_wrapper.pack_end(&signed_toggler, false, false, 0);

    automation_container.attach(&signed_toggler_wrapper, 1, 5, 1, 1);

    connect!(
        relm,
        &numerator_spinner,
        connect_value_changed(numerator_spinner),
        {
            Some(AutomationViewMsg::SetNumerator(
                target_dimension,
                numerator_spinner.get_value().max(1.0),
            ))
        }
    );

    connect!(
        relm,
        denominator_spinner.clone(),
        connect_value_changed(denominator_spinner),
        {
            Some(AutomationViewMsg::SetDenominator(
                target_dimension,
                denominator_spinner.get_value().max(1.0),
            ))
        }
    );

    connect!(
        relm,
        &phase_spinner,
        connect_value_changed(phase_spinner),
        {
            Some(AutomationViewMsg::SetPhase(
                target_dimension,
                phase_spinner.get_value(),
            ))
        }
    );

    connect!(
        relm,
        &amplitude_spinner,
        connect_value_changed(amplitude_spinner),
        {
            Some(AutomationViewMsg::SetAmplitude(
                target_dimension,
                amplitude_spinner.get_value(),
            ))
        }
    );

    connect!(
        relm,
        &signed_toggler,
        connect_property_active_notify(signed_toggler),
        {
            Some(AutomationViewMsg::SetSigned(
                target_dimension,
                signed_toggler.get_active(),
            ))
        }
    );

    connect!(
        relm,
        &lfo_type_chooser,
        connect_changed(lfo_type_chooser),
        {
            Some(AutomationViewMsg::SetType(
                target_dimension,
                lfo_type_chooser
                    .get_active_id()
                    .unwrap()
                    .as_str()
                    .to_owned(),
            ))
        }
    );

    automation_container
}

fn build_add_lfo_button(
    relm: &Relm<AutomationView>,
    variable_dimension_count: usize,
    variable_range: &DataRange,
    automation_wrapper: &gtk::Box,
    automation_button_label: &Label,
) -> Button {
    let relm = relm.clone();
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

        let default_lfo = Lfo {
            lfo_type: LfoType::Sine,
            numerator: 1.0,
            denominator: 8.0,
            phase: 0.0,
            amplitude: 0.0,
            signed: false,
        };

        let new_automation = match variable_dimension_count {
            1 => {
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 0, default_lfo));
                Automation::Lfo(default_lfo)
            }
            2 => {
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 0, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 1, default_lfo));
                Automation::Lfo2d(default_lfo, default_lfo)
            }
            3 => {
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 0, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 1, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 2, default_lfo));
                Automation::Lfo3d(default_lfo, default_lfo, default_lfo)
            }
            4 => {
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 0, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 1, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 2, default_lfo));
                automation_wrapper.add(&build_lfo_row(&relm, &variable_range, 3, default_lfo));
                Automation::Lfo4d(default_lfo, default_lfo, default_lfo, default_lfo)
            }
            _ => Automation::None,
        };

        relm.stream()
            .emit(AutomationViewMsg::SetAutomation(new_automation));

        automation_button_label.set_text("LFO");
        add_lfo_button.set_label("Replace LFO");

        automation_wrapper.show_all();
    });

    add_lfo_button
}

pub struct AutomationViewModel {
    parent_relm: Relm<RenderStageConfigView>,
    variable_name: String,
    dimension_count: usize,
    variable_range: DataRange,
    config: Automation,
}

pub struct AutomationView {
    model: AutomationViewModel,
    relm: Relm<Self>,
    root: MenuButton,
}

impl Update for AutomationView {
    type Model = AutomationViewModel;
    type ModelParam = (
        Relm<RenderStageConfigView>,
        String,
        usize,
        DataRange,
        Automation,
    );
    type Msg = AutomationViewMsg;

    fn model(
        _: &Relm<Self>,
        model: (
            Relm<RenderStageConfigView>,
            String,
            usize,
            DataRange,
            Automation,
        ),
    ) -> Self::Model {
        AutomationViewModel {
            parent_relm: model.0,
            variable_name: model.1,
            dimension_count: model.2,
            variable_range: model.3,
            config: model.4,
        }
    }

    fn update(&mut self, event: AutomationViewMsg) {
        match event {
            AutomationViewMsg::SetAutomation(automation_config) => {
                self.model.config = automation_config;
                return;
            }
            _ => (),
        }
        match &mut self.model.config {
            Automation::Lfo(ref mut lfo) => event.update_lfo(lfo),
            Automation::Lfo2d(ref mut lfo_x, ref mut lfo_y) => match event.get_target_index() {
                Some(0) => event.update_lfo(lfo_x),
                Some(1) => event.update_lfo(lfo_y),
                _ => {
                    return;
                }
            },
            Automation::Lfo3d(ref mut lfo_x, ref mut lfo_y, ref mut lfo_z) => {
                match event.get_target_index() {
                    Some(0) => event.update_lfo(lfo_x),
                    Some(1) => event.update_lfo(lfo_y),
                    Some(2) => event.update_lfo(lfo_z),
                    _ => {
                        return;
                    }
                }
            }
            Automation::Lfo4d(ref mut lfo_x, ref mut lfo_y, ref mut lfo_z, ref mut lfo_w) => {
                match event.get_target_index() {
                    Some(0) => event.update_lfo(lfo_x),
                    Some(1) => event.update_lfo(lfo_y),
                    Some(2) => event.update_lfo(lfo_z),
                    Some(3) => event.update_lfo(lfo_w),
                    _ => {
                        return;
                    }
                }
            }
            Automation::None => {
                return;
            }
        }

        self.model
            .parent_relm
            .stream()
            .emit(RenderStageConfigViewMsg::UpdateVariableAutomation(
                self.model.variable_name.clone(),
                self.model.config.clone(),
            ));
    }
}

impl Widget for AutomationView {
    type Root = MenuButton;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let button_label = match model.config {
            Automation::None => emoji::objects::tool::GEAR,
            Automation::Lfo(_)
            | Automation::Lfo2d(_, _)
            | Automation::Lfo3d(_, _, _)
            | Automation::Lfo4d(_, _, _, _) => "LFO",
        };

        let automation_button_label = Label::new(Some(button_label));
        let automation_button = MenuButton::new();
        automation_button.add(&automation_button_label);

        let automation_popover = Popover::new(Some(&automation_button));
        automation_button.set_popover(Some(&automation_popover));

        let automation_wrapper = gtk::Box::new(Vertical, 4);
        automation_wrapper.set_property_width_request(320);

        let mut has_automation = false;
        match model.config {
            Automation::Lfo(lfo) => {
                if model.dimension_count == 1 {
                    has_automation = true;
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 0, lfo));
                }
            }
            Automation::Lfo2d(lfo_x, lfo_y) => {
                if model.dimension_count == 2 {
                    has_automation = true;
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 0, lfo_x));

                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 1, lfo_y));
                }
            }
            Automation::Lfo3d(lfo_x, lfo_y, lfo_z) => {
                if model.dimension_count == 3 {
                    has_automation = true;
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 0, lfo_x));
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 1, lfo_y));
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 2, lfo_z));
                }
            }
            Automation::Lfo4d(lfo_x, lfo_y, lfo_z, lfo_w) => {
                if model.dimension_count == 4 {
                    has_automation = true;
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 0, lfo_x));
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 1, lfo_y));
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 2, lfo_z));
                    automation_wrapper.add(&build_lfo_row(relm, &model.variable_range, 3, lfo_w));
                }
            }
            Automation::None => (),
        }

        if !has_automation {
            automation_wrapper.add(&build_add_lfo_button(
                relm,
                model.dimension_count,
                &model.variable_range,
                &automation_wrapper,
                &automation_button_label,
            ))
        }

        automation_popover.add(&automation_wrapper);
        automation_wrapper.show_all();

        Self {
            relm: relm.clone(),
            model,
            root: automation_button,
        }
    }
}
