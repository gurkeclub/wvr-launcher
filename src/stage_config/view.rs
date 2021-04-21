use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;

use glib::Cast;

use gtk::prelude::{GtkListStoreExtManual, TreeSortableExtManual};
use gtk::Orientation::{self, Horizontal, Vertical};
use gtk::{
    Adjustment, ComboBoxExt, ComboBoxText, ComboBoxTextExt, ContainerExt, EditableSignals, Entry,
    EntryExt, Grid, GridExt, GtkListStoreExt, Label, LabelExt, OrientableExt, PolicyType,
    ScrolledWindow, ScrolledWindowExt, Separator, SortColumn, SortType, WidgetExt,
};

use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

use strsim::levenshtein;

use wvr_data::config::project_config::{
    BufferPrecision, FilterMode, RenderStageConfig, SampledInput,
};
use wvr_data::{DataHolder, DataRange};

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;

use super::input;
use super::variable;

use super::list_store_sort_function;
use super::load_available_filter_list;

#[derive(Msg)]
pub enum RenderStageConfigViewMsg {
    SetName(String),
    SetFilter(String),
    SetPrecision(String),

    UpdateInputList,
    UpdateVariable(String, DataHolder),
    UpdateInputChoiceList(Vec<String>),
}

pub struct RenderStageConfigViewModel {
    parent_relm: Relm<ConfigPanel>,
    id: Uuid,
    project_path: PathBuf,
    config: RenderStageConfig,
    input_choice_list: Vec<String>,
}
pub struct RenderStageConfigView {
    model: RenderStageConfigViewModel,
    relm: Relm<Self>,
    root: gtk::Box,

    filter_mode_params_label: Label,
    filter_mode_params_container: gtk::Box,

    filter_config_container: Grid,
    input_widget_list: HashMap<String, (ComboBoxText, ComboBoxText)>,
}

impl RenderStageConfigView {
    pub fn update_input_list(&mut self) {
        self.model.config.inputs.clear();
        for (input_name, (input_type_entry, input_name_chooser)) in self.input_widget_list.iter() {
            let input_value = match input_type_entry.get_active_id().unwrap().as_str() {
                "Linear" => SampledInput::Linear(
                    input_name_chooser
                        .get_active_id()
                        .unwrap_or_else(|| glib::GString::from(""))
                        .to_string(),
                ),
                "Nearest" => SampledInput::Nearest(
                    input_name_chooser
                        .get_active_id()
                        .unwrap_or_else(|| glib::GString::from(""))
                        .to_string(),
                ),
                "Mipmaps" => SampledInput::Mipmaps(
                    input_name_chooser
                        .get_active_id()
                        .unwrap_or_else(|| glib::GString::from(""))
                        .to_string(),
                ),
                _ => unreachable!(),
            };

            self.model
                .parent_relm
                .stream()
                .emit(ConfigPanelMsg::UpdateRenderStageInput(
                    self.model.id,
                    input_name.clone(),
                    input_value.clone(),
                ));

            self.model
                .config
                .inputs
                .insert(input_name.clone(), input_value);
        }
    }

    pub fn update_filter_params(&mut self, value: DataHolder) {
        match &mut self.model.config.filter_mode_params {
            FilterMode::Rectangle(x_a, y_a, x_b, y_b) => {
                if let DataHolder::Float4(params) = value {
                    *x_a = params[0];
                    *y_a = params[1];
                    *x_b = params[2];
                    *y_b = params[3];
                }
            }
            FilterMode::Particles(count) => {
                if let DataHolder::Int(new_count) = value {
                    *count = new_count as usize;
                }
            }
        }

        self.model
            .parent_relm
            .stream()
            .emit(ConfigPanelMsg::UpdateRenderStageFilterModeParams(
                self.model.id,
                self.model.config.filter_mode_params.clone(),
            ));
    }

    pub fn update_input_choice_list(&mut self, input_choice_list: &[String]) {
        let input_choice_list = input_choice_list.to_vec();
        if input_choice_list == self.model.input_choice_list {
            return;
        }

        self.model.input_choice_list = input_choice_list;

        for (_, (_, input_name_chooser)) in self.input_widget_list.iter() {
            let new_id;

            if let Some(current_id) = input_name_chooser.get_active_id() {
                let current_id = current_id.to_string();

                if let Some(default_id) = self.model.input_choice_list.get(0) {
                    let mut closest_id = default_id.clone();
                    let mut closest_id_distance = levenshtein(&current_id, &closest_id);

                    for name in &self.model.input_choice_list {
                        let candidate_id_distance = levenshtein(&current_id, &name);

                        if candidate_id_distance < closest_id_distance {
                            closest_id = name.clone();
                            closest_id_distance = candidate_id_distance;
                        }
                    }

                    new_id = Some(closest_id);
                } else {
                    new_id = None;
                }
            } else if let Some(default_id) = self.model.input_choice_list.get(0) {
                new_id = Some(default_id.clone());
            } else {
                new_id = None;
            }

            let input_name_store = input_name_chooser
                .get_model()
                .unwrap()
                .downcast::<gtk::ListStore>()
                .unwrap();
            input_name_store.clear();

            for name in &self.model.input_choice_list {
                input_name_store.insert_with_values(None, &[0, 1], &[name, name]);
            }

            if let Some(new_id) = new_id {
                input_name_chooser.set_active_id(Some(&new_id));
            } else {
                input_name_chooser.set_active_id(None);
            }
        }
        self.update_input_list();
    }

    pub fn set_filter(&mut self, filter_name: &str) {
        self.model.config.filter = filter_name.to_string();
        if let Some((_, filter_config)) = &load_available_filter_list(&self.model.project_path)
            .unwrap()
            .get(filter_name)
        {
            self.model.config.filter_mode_params = filter_config.mode.clone();

            self.filter_mode_params_container = match self.model.config.filter_mode_params {
                FilterMode::Rectangle(_, _, _, _) => {
                    self.filter_mode_params_label.set_text("");
                    gtk::Box::new(Horizontal, 0)
                }
                FilterMode::Particles(count) => {
                    self.filter_mode_params_label.set_text("Particle count: ");
                    variable::create_int_spinner(
                        &self.relm,
                        "_FILTER_MODE_PARAMS",
                        count as i64,
                        &DataRange::IntRange(1, 1_000_000, 1),
                    )
                }
            };

            let old_inputs = self.model.config.inputs.clone();

            self.model.config.inputs.clear();
            self.input_widget_list.clear();

            for children in &self.filter_config_container.get_children() {
                self.filter_config_container.remove(children);
            }

            let default_input = SampledInput::Linear(
                self.model
                    .input_choice_list
                    .get(0)
                    .map(String::clone)
                    .unwrap_or_default(),
            );

            let mut uniform_name_list = filter_config.inputs.clone();
            uniform_name_list.sort();
            for (input_index, uniform_name) in uniform_name_list.iter().enumerate() {
                let input_value = old_inputs.get(uniform_name).unwrap_or(&default_input);

                let input_name_label = Label::new(Some(uniform_name));
                input_name_label.set_xalign(0.0);

                let (input_wrapper, input_type_chooser, input_name_chooser) =
                    input::build_input_row(&self.relm, &self.model.input_choice_list, &input_value);

                self.model
                    .config
                    .inputs
                    .insert(uniform_name.clone(), input_value.clone());

                self.filter_config_container
                    .attach(&input_name_label, 0, input_index as i32, 1, 1);
                self.filter_config_container
                    .attach(&input_wrapper, 1, input_index as i32, 1, 1);

                self.input_widget_list.insert(
                    uniform_name.clone(),
                    (input_type_chooser, input_name_chooser),
                );
            }

            let old_variables = self.model.config.variables.clone();

            self.model.config.variables.clear();

            let mut variable_name_list: Vec<String> =
                filter_config.variables.keys().map(String::clone).collect();
            variable_name_list.sort();

            for (variable_index, variable_name) in variable_name_list.iter().enumerate() {
                let (default_value, value_range) =
                    filter_config.variables.get(variable_name).unwrap();
                let variable_value = old_variables.get(variable_name).unwrap_or(default_value);

                let variable_wrapper = variable::build_variable_row(
                    &self.relm,
                    variable_name,
                    &variable_value,
                    value_range,
                );

                let variable_name_label = Label::new(Some(variable_name));
                variable_name_label.set_xalign(0.0);

                self.filter_config_container.attach(
                    &variable_name_label,
                    0,
                    (uniform_name_list.len() + variable_index) as i32,
                    1,
                    1,
                );
                self.filter_config_container.attach(
                    &variable_wrapper,
                    1,
                    (uniform_name_list.len() + variable_index) as i32,
                    1,
                    1,
                );

                self.model
                    .config
                    .variables
                    .insert(variable_name.clone(), variable_value.clone());
            }

            self.filter_config_container.show_all();
        }
        self.model
            .parent_relm
            .stream()
            .emit(ConfigPanelMsg::UpdateRenderStageFilter(
                self.model.id,
                self.model.config.filter.clone(),
            ));

        for (variable_name, variable_value) in &self.model.config.variables {
            self.model
                .parent_relm
                .stream()
                .emit(ConfigPanelMsg::UpdateRenderStageVariable(
                    self.model.id,
                    variable_name.clone(),
                    variable_value.clone(),
                ));
        }
    }
}

impl Update for RenderStageConfigView {
    type Model = RenderStageConfigViewModel;
    type ModelParam = (
        Uuid,
        PathBuf,
        RenderStageConfig,
        Vec<String>,
        Relm<ConfigPanel>,
    );
    type Msg = RenderStageConfigViewMsg;

    fn model(
        _: &Relm<Self>,
        model: (
            Uuid,
            PathBuf,
            RenderStageConfig,
            Vec<String>,
            Relm<ConfigPanel>,
        ),
    ) -> Self::Model {
        RenderStageConfigViewModel {
            id: model.0,
            project_path: model.1,
            config: model.2,
            input_choice_list: model.3,
            parent_relm: model.4,
        }
    }

    fn update(&mut self, event: RenderStageConfigViewMsg) {
        match event {
            RenderStageConfigViewMsg::SetName(new_name) => {
                self.model
                    .parent_relm
                    .stream()
                    .emit(ConfigPanelMsg::UpdateRenderStageName(
                        self.model.id,
                        new_name,
                    ));
            }
            RenderStageConfigViewMsg::SetFilter(new_filter) => {
                self.set_filter(&new_filter);
            }
            RenderStageConfigViewMsg::SetPrecision(new_precision) => {
                let new_precision = match new_precision.as_str() {
                    "U8" => BufferPrecision::U8,
                    "F16" => BufferPrecision::F16,
                    "F32" => BufferPrecision::F32,
                    _ => unreachable!(),
                };
                self.model.config.precision = new_precision;
            }
            RenderStageConfigViewMsg::UpdateInputList => self.update_input_list(),
            RenderStageConfigViewMsg::UpdateVariable(name, value) => {
                if name == "_FILTER_MODE_PARAMS" {
                    self.update_filter_params(value);
                } else {
                    self.model.parent_relm.stream().emit(
                        ConfigPanelMsg::UpdateRenderStageVariable(self.model.id, name, value),
                    );
                }
            }
            RenderStageConfigViewMsg::UpdateInputChoiceList(choice_list) => {
                self.update_input_choice_list(&choice_list);
            }
        }
    }
}

impl Widget for RenderStageConfigView {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let mut input_widget_list = HashMap::new();

        let root = gtk::Box::new(Vertical, 4);
        root.set_property_margin(8);

        let base_config = gtk::Grid::new();
        base_config.set_hexpand(true);
        base_config.set_row_spacing(4);
        base_config.set_column_spacing(8);
        base_config.set_orientation(Orientation::Vertical);

        // Building of the input name row

        //let name_label = Label::new(Some("Name: "));
        //name_label.set_xalign(0.0);

        let name_entry = Entry::new();
        name_entry.set_hexpand(true);
        name_entry.set_text(&model.config.name);
        connect!(
            relm,
            name_entry,
            connect_changed(val),
            Some(RenderStageConfigViewMsg::SetName(
                val.get_text().to_string()
            ))
        );

        // Building of the filter selection row
        let available_filters = load_available_filter_list(&model.project_path).unwrap();
        let filter_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in available_filters.keys() {
            filter_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        filter_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);

        let filter_chooser = gtk::ComboBoxText::new();
        filter_chooser.set_hexpand(true);
        filter_chooser.set_model(Some(&filter_store));

        filter_chooser.set_id_column(0);
        filter_chooser.set_entry_text_column(1);

        {
            let filter_chooser = filter_chooser.clone();
            connect!(
                relm,
                filter_chooser,
                connect_changed(val),
                Some(RenderStageConfigViewMsg::SetFilter(
                    val.get_active_text().unwrap().to_string()
                ))
            );
        }

        // Building of the filter_mode_params selection row
        let filter_mode_params_label = Label::new(None);
        filter_mode_params_label.set_xalign(0.0);

        let filter_mode_params_container = match model.config.filter_mode_params {
            FilterMode::Rectangle(_, _, _, _) => {
                filter_mode_params_label.set_text("");
                gtk::Box::new(Horizontal, 0)
            }
            FilterMode::Particles(count) => {
                filter_mode_params_label.set_text("Particle count: ");
                variable::create_int_spinner(
                    relm,
                    "_FILTER_MODE_PARAMS",
                    count as i64,
                    &DataRange::IntRange(1, 1_000_000, 1),
                )
            }
        };

        // Building of the precision selection row
        //let precision_label = Label::new(Some("Precision: "));
        //precision_label.set_xalign(0.0);

        let available_precisions = ["U8", "F16", "F32"];
        let precision_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in available_precisions.iter() {
            precision_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        precision_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
        precision_store.set_default_sort_func(&list_store_sort_function);

        let precision_chooser = gtk::ComboBoxText::new();
        precision_chooser.set_hexpand(false);
        precision_chooser.set_model(Some(&precision_store));

        precision_chooser.set_id_column(0);
        precision_chooser.set_entry_text_column(1);

        match model.config.precision {
            BufferPrecision::U8 => precision_chooser.set_active_id(Some("U8")),
            BufferPrecision::F16 => precision_chooser.set_active_id(Some("F16")),
            BufferPrecision::F32 => precision_chooser.set_active_id(Some("F32")),
        };

        {
            let precision_chooser = precision_chooser.clone();
            connect!(
                relm,
                &precision_chooser,
                connect_changed(val),
                Some(RenderStageConfigViewMsg::SetPrecision(
                    val.get_active_text().unwrap().to_string()
                ))
            );
        }

        //base_config.attach(&name_label, 0, 0, 1, 1);
        base_config.attach(&name_entry, 0, 0, 2, 1);

        //base_config.attach(&filter_label, 0, 2, 1, 1);
        base_config.attach(&filter_chooser, 0, 1, 1, 1);
        //base_config.attach(&precision_label, 0, 1, 1, 1);
        base_config.attach(&precision_chooser, 1, 1, 1, 1);

        base_config.attach(&filter_mode_params_label, 0, 2, 1, 1);
        base_config.attach(&filter_mode_params_container, 1, 2, 1, 1);

        let filter_config_panel = gtk::Box::new(Vertical, 16);

        let filter_config_container = gtk::Grid::new();
        filter_config_container.set_row_spacing(4);
        filter_config_container.set_column_spacing(8);
        filter_config_container.set_orientation(Orientation::Vertical);

        if available_filters.contains_key(&model.config.filter) {
            let filter_config = available_filters
                .get(&model.config.filter)
                .unwrap()
                .1
                .clone();

            filter_chooser.set_active_id(Some(&model.config.filter));

            let mut input_name_list = filter_config.inputs.clone();
            input_name_list.sort();
            for (input_index, input_name) in input_name_list.iter().enumerate() {
                let input_value = if let Some(input_value) = model.config.inputs.get(input_name) {
                    input_value.clone()
                } else {
                    SampledInput::Linear("".to_string())
                };

                let input_name_label = Label::new(Some(input_name));
                input_name_label.set_xalign(0.0);

                let (input_wrapper, input_type_entry, input_value_entry) =
                    input::build_input_row(relm, &model.input_choice_list, &input_value);

                filter_config_container.add(&input_wrapper);

                filter_config_container.attach(&input_name_label, 0, input_index as i32, 1, 1);
                filter_config_container.attach(&input_wrapper, 1, input_index as i32, 1, 1);

                input_widget_list.insert(input_name.clone(), (input_type_entry, input_value_entry));
            }

            let mut variable_name_list: Vec<String> =
                filter_config.variables.keys().map(String::clone).collect();
            variable_name_list.sort();

            for (variable_index, variable_name) in variable_name_list.iter().enumerate() {
                let (default_value, value_range) =
                    filter_config.variables.get(variable_name).unwrap();
                let variable_value =
                    if let Some(stage_variable_value) = model.config.variables.get(variable_name) {
                        stage_variable_value
                    } else {
                        default_value
                    };
                let variable_wrapper = variable::build_variable_row(
                    relm,
                    &variable_name,
                    &variable_value,
                    &value_range,
                );

                let variable_name_label = Label::new(Some(variable_name));
                variable_name_label.set_xalign(0.0);

                filter_config_container.attach(
                    &variable_name_label,
                    0,
                    (input_name_list.len() + variable_index) as i32,
                    1,
                    1,
                );
                filter_config_container.attach(
                    &variable_wrapper,
                    1,
                    (input_name_list.len() + variable_index) as i32,
                    1,
                    1,
                );
            }
        }

        filter_config_panel.add(&filter_config_container);

        let filter_config_wrapper = ScrolledWindow::new::<Adjustment, Adjustment>(None, None);

        filter_config_wrapper.set_policy(PolicyType::Never, PolicyType::Automatic);
        filter_config_wrapper.set_hexpand(true);
        filter_config_wrapper.set_vexpand(true);
        filter_config_wrapper.add(&filter_config_panel);

        root.add(&base_config);
        root.add(&Separator::new(Horizontal));
        root.add(&filter_config_wrapper);

        Self {
            relm: relm.clone(),
            model,
            root,

            filter_mode_params_label,
            filter_mode_params_container,

            filter_config_container,
            input_widget_list,
        }
    }
}
