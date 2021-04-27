use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;

use glib::{Cast, ToValue};

//use gtk::prelude::*, };
use gtk::{
    prelude::{GtkListStoreExtManual, TreeSortableExtManual, TreeStoreExtManual},
    Adjustment, CellLayoutExt, CellRendererText, ComboBoxExt, ComboBoxText, ComboBoxTextExt,
    ContainerExt, EditableSignals, Entry, EntryExt, Grid, GridExt, GtkListStoreExt, Label,
    LabelExt, MenuButton, MenuButtonExt, OrientableExt,
    Orientation::{self, Horizontal, Vertical},
    PolicyType, Popover, ScrolledWindow, ScrolledWindowExt, Separator, SortColumn, SortType,
    TreeModelExt, TreeSelectionExt, TreeStoreExt, TreeViewColumn, TreeViewExt, WidgetExt,
};

use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

use strsim::levenshtein;

use wvr_data::config::project_config::{
    BufferPrecision, FilterConfig, FilterMode, RenderStageConfig, SampledInput,
};
use wvr_data::{DataHolder, DataRange};

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;

use super::input;
use super::variable;

use super::list_store_sort_function;

#[derive(Msg)]
pub enum RenderStageConfigViewMsg {
    SetName(String),
    SetFilter(String),
    SetPrecision(String),

    UpdateInput(String, SampledInput),
    UpdateVariable(String, DataHolder),
    UpdateInputChoiceList(Vec<String>),
}

pub struct RenderStageConfigViewModel {
    parent_relm: Relm<ConfigPanel>,
    id: Uuid,
    config: RenderStageConfig,
    input_choice_list: Vec<String>,

    available_filter_list: HashMap<String, (PathBuf, FilterConfig, bool)>,
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

        for (uniform_name, (_, input_name_chooser)) in self.input_widget_list.iter() {
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

                if let Some(input_value) = self.model.config.inputs.get(uniform_name) {
                    self.relm
                        .stream()
                        .emit(RenderStageConfigViewMsg::UpdateInput(
                            uniform_name.clone(),
                            match input_value {
                                SampledInput::Linear(_) => SampledInput::Linear(new_id),
                                SampledInput::Mipmaps(_) => SampledInput::Linear(new_id),
                                SampledInput::Nearest(_) => SampledInput::Linear(new_id),
                            },
                        ));
                }
            } else {
                input_name_chooser.set_active_id(None);
            }
        }
    }

    pub fn set_filter(&mut self, filter_name: &str) {
        self.model.config.filter = filter_name.to_string();
        if let Some((_, filter_config, _)) = &self.model.available_filter_list.get(filter_name) {
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
                    input::build_input_row(
                        &self.relm,
                        &self.model.input_choice_list,
                        &uniform_name,
                        &input_value,
                    );

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
        RenderStageConfig,
        Vec<String>,
        HashMap<String, (PathBuf, FilterConfig, bool)>,
        Relm<ConfigPanel>,
    );
    type Msg = RenderStageConfigViewMsg;

    fn model(
        _: &Relm<Self>,
        model: (
            Uuid,
            RenderStageConfig,
            Vec<String>,
            HashMap<String, (PathBuf, FilterConfig, bool)>,
            Relm<ConfigPanel>,
        ),
    ) -> Self::Model {
        RenderStageConfigViewModel {
            id: model.0,
            config: model.1,
            input_choice_list: model.2,
            available_filter_list: model.3,
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
                self.model.config.precision = new_precision.clone();
                self.model
                    .parent_relm
                    .stream()
                    .emit(ConfigPanelMsg::UpdateRenderStagePrecision(
                        self.model.id,
                        new_precision,
                    ));
            }
            RenderStageConfigViewMsg::UpdateInput(input_name, input_value) => {
                self.model
                    .parent_relm
                    .stream()
                    .emit(ConfigPanelMsg::UpdateRenderStageInput(
                        self.model.id,
                        input_name.clone(),
                        input_value.clone(),
                    ));

                self.model.config.inputs.insert(input_name, input_value);
            }
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
        let root = gtk::Box::new(Vertical, 4);
        root.set_property_margin(8);

        let base_config = gtk::Box::new(Horizontal, 8);

        // Building of the input name widget
        let name_entry = Entry::new();
        name_entry.set_hexpand(true);
        name_entry.set_text(&model.config.name);
        name_entry.set_tooltip_text(Some("Layer name"));
        connect!(
            relm,
            name_entry,
            connect_changed(val),
            Some(RenderStageConfigViewMsg::SetName(
                val.get_text().to_string()
            ))
        );

        // Building of the filter selection widget
        let filter_chooser_button =
            build_filter_chooser(relm, &model.available_filter_list, &model.config.filter);
        filter_chooser_button.set_tooltip_text(Some("Used filter"));

        // Building of the filter_mode_params selection widget
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

        // Building of the precision selection widget
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
        precision_chooser.set_tooltip_text(Some("Generated buffer precision"));

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
        base_config.add(&name_entry);

        base_config.add(&filter_chooser_button);
        base_config.add(&precision_chooser);

        base_config.add(&filter_mode_params_label);
        base_config.add(&filter_mode_params_container);

        let (filter_config_container, filter_config_panel, input_widget_list) =
            build_filter_config(relm, &model);

        root.add(&base_config);
        root.add(&Separator::new(Horizontal));
        root.add(&filter_config_panel);

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

pub fn build_filter_config(
    relm: &Relm<RenderStageConfigView>,
    model: &RenderStageConfigViewModel,
) -> (
    Grid,
    ScrolledWindow,
    HashMap<String, (ComboBoxText, ComboBoxText)>,
) {
    let mut input_widget_list = HashMap::new();

    let filter_config_panel = gtk::Box::new(Vertical, 16);
    let filter_config_container = gtk::Grid::new();
    filter_config_container.set_row_spacing(4);
    filter_config_container.set_column_spacing(8);
    filter_config_container.set_orientation(Orientation::Vertical);

    if model
        .available_filter_list
        .contains_key(&model.config.filter)
    {
        let filter_config = model
            .available_filter_list
            .get(&model.config.filter)
            .unwrap()
            .1
            .clone();

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
                input::build_input_row(relm, &model.input_choice_list, &input_name, &input_value);

            //filter_config_container.add(&input_wrapper);

            filter_config_container.attach(&input_name_label, 0, input_index as i32, 1, 1);
            filter_config_container.attach(&input_wrapper, 1, input_index as i32, 2, 1);

            input_widget_list.insert(input_name.clone(), (input_type_entry, input_value_entry));
        }

        let mut variable_name_list: Vec<String> =
            filter_config.variables.keys().map(String::clone).collect();
        variable_name_list.sort();

        for (variable_index, variable_name) in variable_name_list.iter().enumerate() {
            let (default_value, value_range) = filter_config.variables.get(variable_name).unwrap();
            let variable_value =
                if let Some(stage_variable_value) = model.config.variables.get(variable_name) {
                    stage_variable_value
                } else {
                    default_value
                };
            let variable_wrapper =
                variable::build_variable_row(relm, &variable_name, &variable_value, &value_range);

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

    (
        filter_config_container,
        filter_config_wrapper,
        input_widget_list,
    )
}

pub fn build_filter_chooser(
    relm: &Relm<RenderStageConfigView>,
    available_filter_list: &HashMap<String, (PathBuf, FilterConfig, bool)>,
    selected_filter: &str,
) -> MenuButton {
    let filter_chooser_label = Label::new(Some(selected_filter));
    let filter_chooser_button = MenuButton::new();
    filter_chooser_button.add(&filter_chooser_label);

    let filter_chooser_popover = Popover::new(Some(&filter_chooser_button));
    filter_chooser_button.set_popover(Some(&filter_chooser_popover));

    let filter_store = gtk::TreeStore::new(&[glib::Type::String, glib::Type::String]);

    let mut parents_iter = HashMap::new();

    //let root = filter_store.append(None);
    for name in available_filter_list.keys() {
        let mut parent_chain = String::new();

        let filter_name = name.split('/').last().unwrap();
        for sub_name in name.split('/') {
            let sub_name = sub_name.to_owned();

            if sub_name == filter_name {
                if let Some(direct_parent) = parents_iter.get(&parent_chain) {
                    filter_store.insert_with_values(
                        Some(direct_parent),
                        None,
                        &[0, 1],
                        &[&sub_name, name],
                    );
                } else {
                    filter_store.insert_with_values(None, None, &[0, 1], &[&sub_name, name]);
                }
            } else {
                let old_parent_chain = parent_chain.clone();

                if !parent_chain.is_empty() {
                    parent_chain.push('/');
                }
                parent_chain.push_str(&sub_name);

                if parents_iter.get(&parent_chain).is_none() {
                    let sub_name_iter =
                        if let Some(direct_parent) = parents_iter.get(&old_parent_chain) {
                            filter_store.append(Some(direct_parent))
                        } else {
                            filter_store.append(None)
                        };

                    filter_store.set_value(&sub_name_iter, 0, &sub_name.to_value());

                    parents_iter.insert(sub_name.clone(), sub_name_iter);
                }
            }
        }
    }
    filter_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);

    let filter_chooser = gtk::TreeView::new();

    filter_chooser.set_hexpand(true);
    filter_chooser.set_model(Some(&filter_store));

    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    filter_chooser.append_column(&column);

    filter_chooser.show_all();
    for children in &filter_chooser_popover.get_children() {
        filter_chooser_popover.remove(children);
    }
    filter_chooser_popover.add(&filter_chooser);

    connect!(relm, filter_chooser, connect_cursor_changed(val), {
        if let Some((list_model, iter)) = val.get_selection().get_selected() {
            if let Some(filter_name) = list_model
                .get_value(&iter, 1)
                .get::<String>()
                .ok()
                .and_then(|value| value)
            {
                filter_chooser_label.set_text(filter_name.as_str());
                Some(RenderStageConfigViewMsg::SetFilter(filter_name))
            } else {
                None
            }
        } else {
            None
        }
    });

    filter_chooser_button
}
