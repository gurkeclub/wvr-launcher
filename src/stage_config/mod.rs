use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Result;

use uuid::Uuid;

use glib::Cast;

use gdk::RGBA;
use gtk::prelude::{GtkListStoreExtManual, NotebookExtManual, TreeSortableExtManual};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, Button, ButtonExt, ComboBoxExt, ComboBoxText, ContainerExt, EditableSignals, Entry,
    EntryExt, GtkListStoreExt, Label, LabelExt, Notebook, ScrolledWindow, SortColumn, SortType,
    SpinButtonExt, StateFlags, SwitchExt, TreeIter, TreeModel, TreeModelExt, WidgetExt,
};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;

use strsim::levenshtein;

use wvr_data::config::project_config::{
    BufferPrecision, FilterConfig, RenderStageConfig, SampledInput,
};
use wvr_data::DataHolder;

pub mod input;
pub mod variable;

use variable::VariableInput;

pub fn build_list_view(
    relm: &Relm<crate::Win>,
    project_path: &Path,
    render_stage_config_list: &[RenderStageConfig],
    input_choice_list: &[String],
    render_stage_config_widget_list: &mut HashMap<
        Uuid,
        (
            RenderStageConfig,
            Component<RenderStageConfigView>,
            gtk::Box,
        ),
    >,
    render_stage_order: &mut Vec<Uuid>,
) -> (gtk::Box, gtk::Box) {
    let render_stage_list_panel = gtk::Box::new(Vertical, 2);
    render_stage_list_panel.set_property_margin(4);

    let render_stage_list_control_container = gtk::Box::new(Horizontal, 8);
    render_stage_list_control_container.set_property_margin(8);

    let add_render_stage_button = Button::new();
    add_render_stage_button.set_label("Add Render Stage");
    add_render_stage_button.set_hexpand(true);
    connect!(
        relm,
        add_render_stage_button,
        connect_clicked(_),
        Some(crate::Msg::AddRenderStage)
    );

    render_stage_list_control_container.add(&add_render_stage_button);

    let render_stage_list_container = gtk::Box::new(Vertical, 16);
    render_stage_list_container.set_property_margin(8);

    for render_stage_config in render_stage_config_list.iter() {
        let (id, wrapper, render_stage_config_view) = build_render_stage_config_row(
            relm,
            project_path,
            &render_stage_config,
            &input_choice_list,
        );
        render_stage_list_container.add(&wrapper);
        render_stage_config_widget_list.insert(
            id,
            (
                render_stage_config.clone(),
                render_stage_config_view,
                wrapper,
            ),
        );

        render_stage_order.push(id);
    }

    let render_stage_list_container_wrapper = ScrolledWindow::new(
        Some(&Adjustment::new(320.0, 320.0, 10000.0, 1.0, 1.0, 1.0)),
        Some(&Adjustment::new(320.0, 320.0, 100000.0, 0.0, 0.0, 1.0)),
    );
    render_stage_list_container_wrapper.set_size_request(480, 320);
    render_stage_list_container_wrapper.set_hexpand(true);
    render_stage_list_container_wrapper.set_vexpand(true);
    render_stage_list_container_wrapper.add(&render_stage_list_container);

    render_stage_list_panel.add(&render_stage_list_container_wrapper);
    render_stage_list_panel.add(&render_stage_list_control_container);

    (render_stage_list_panel, render_stage_list_container)
}

pub fn build_render_stage_config_row(
    relm: &Relm<crate::Win>,
    project_path: &Path,
    render_stage_config: &RenderStageConfig,
    input_choice_list: &[String],
) -> (Uuid, gtk::Box, Component<RenderStageConfigView>) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 2);

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    connect!(
        relm,
        remove_button,
        connect_clicked(_),
        Some(crate::Msg::RemoveRenderStage(id))
    );

    let render_stage_config_view = wrapper.add_widget::<RenderStageConfigView>((
        id,
        project_path.to_owned(),
        render_stage_config.clone(),
        input_choice_list.to_vec(),
        relm.clone(),
    ));
    wrapper.add(&remove_button);

    (id, wrapper, render_stage_config_view)
}

#[derive(Msg)]
pub enum RenderStageConfigViewMsg {
    SetName(String),
    SetFilter(String),
    SetPrecision(String),

    UpdateInputList,
    UpdateVariableList,
    UpdateInputChoiceList(Vec<String>),
}

pub struct RenderStageConfigViewModel {
    parent_relm: Relm<crate::Win>,
    id: Uuid,
    project_path: PathBuf,
    config: RenderStageConfig,
    input_choice_list: Vec<String>,
}
pub struct RenderStageConfigView {
    model: RenderStageConfigViewModel,
    relm: Relm<Self>,
    root: gtk::Box,

    filter_chooser: ComboBoxText,
    precision_chooser: ComboBoxText,

    input_list_container: gtk::Box,
    input_widget_list: HashMap<String, (gtk::Box, ComboBoxText, ComboBoxText)>,

    variable_list_container: gtk::Box,
    variable_widget_list: HashMap<String, (gtk::Box, VariableInput)>,
}
impl RenderStageConfigView {
    pub fn update_input_list(&mut self) {
        self.model.config.inputs.clear();
        for (input_name, (_, input_type_entry, input_name_chooser)) in self.input_widget_list.iter()
        {
            self.model.config.inputs.insert(
                input_name.clone(),
                match input_type_entry.get_active_id().unwrap().as_str() {
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
                },
            );
        }
    }

    pub fn update_variable_list(&mut self) {
        self.model.config.variables.clear();
        for (variable_name, (_, variable_input)) in self.variable_widget_list.iter() {
            let variable_as_data_holder = match variable_input {
                VariableInput::Bool(bool_switch) => DataHolder::Bool(bool_switch.get_active()),
                VariableInput::Int(int_spinner) => DataHolder::Int(int_spinner.get_value() as i32),
                VariableInput::Float(float_spinner) => {
                    DataHolder::Float(float_spinner.get_value() as f32)
                }
                VariableInput::Vec2(x_spinner, y_spinner) => {
                    DataHolder::Float2([x_spinner.get_value() as f32, y_spinner.get_value() as f32])
                }
                VariableInput::Vec3(x_spinner, y_spinner, z_spinner) => DataHolder::Float3([
                    x_spinner.get_value() as f32,
                    y_spinner.get_value() as f32,
                    z_spinner.get_value() as f32,
                ]),
                VariableInput::Vec4(x_spinner, y_spinner, z_spinner, w_spinner) => {
                    DataHolder::Float4([
                        x_spinner.get_value() as f32,
                        y_spinner.get_value() as f32,
                        z_spinner.get_value() as f32,
                        w_spinner.get_value() as f32,
                    ])
                }
            };
            self.model
                .config
                .variables
                .insert(variable_name.clone(), variable_as_data_holder);
        }
    }

    pub fn update_input_choice_list(&mut self, input_choice_list: &[String]) {
        self.model.input_choice_list = input_choice_list.to_vec();

        for (_, (_, _, input_name_chooser)) in self.input_widget_list.iter() {
            let current_id = if let Some(id) = input_name_chooser.get_active_id() {
                id.to_string()
            } else {
                input_choice_list[0].clone()
            };

            let mut closest_id = input_choice_list[0].clone();
            let mut closest_id_distance = levenshtein(&current_id, &closest_id);

            let input_name_store = input_name_chooser
                .get_model()
                .unwrap()
                .downcast::<gtk::ListStore>()
                .unwrap();
            input_name_store.clear();

            for name in input_choice_list {
                input_name_store.insert_with_values(None, &[0, 1], &[name, name]);

                let candidate_id_distance = levenshtein(&current_id, &name);

                if candidate_id_distance < closest_id_distance {
                    closest_id = name.clone();
                    closest_id_distance = candidate_id_distance;
                }
            }

            input_name_chooser.set_active_id(Some(&closest_id));
        }
    }

    pub fn set_filter(&mut self, filter_name: &str) {
        self.model.config.filter = filter_name.to_string();
        if let Some((_, filter_config)) = &load_available_filter_list(&self.model.project_path)
            .unwrap()
            .get(filter_name)
        {
            let mut input_to_remove_list = Vec::new();
            for (input_name, (input_wrapper, _, _)) in self.input_widget_list.iter() {
                if !filter_config.inputs.contains(input_name) {
                    self.input_list_container.remove(input_wrapper);
                    self.model.config.inputs.remove(input_name);
                    input_to_remove_list.push(input_name.clone())
                }
            }
            for input_name in input_to_remove_list {
                self.input_widget_list.remove(&input_name);
            }

            let mut input_to_insert_list = Vec::new();
            for uniform_name in &filter_config.inputs {
                if !self.input_widget_list.contains_key(uniform_name) {
                    let input_value = SampledInput::Linear(self.model.input_choice_list[0].clone());

                    let (input_wrapper, input_type_chooser, input_name_chooser) =
                        input::build_input_row(
                            &self.relm,
                            &self.model.input_choice_list,
                            uniform_name,
                            &input_value,
                        );

                    self.model
                        .config
                        .inputs
                        .insert(uniform_name.clone(), input_value);

                    self.input_list_container.add(&input_wrapper);

                    input_to_insert_list.push((
                        uniform_name.clone(),
                        (input_wrapper, input_type_chooser, input_name_chooser),
                    ));
                }
            }

            self.input_widget_list.extend(input_to_insert_list);
            self.input_list_container.show_all();

            let mut variable_to_remove_list = Vec::new();
            for (variable_name, (variable_wrapper, _)) in self.variable_widget_list.iter() {
                if !filter_config.variables.contains_key(variable_name) {
                    self.variable_list_container.remove(variable_wrapper);
                    self.model.config.variables.remove(variable_name);
                    variable_to_remove_list.push(variable_name.clone())
                }
            }
            for variable_name in variable_to_remove_list {
                self.variable_widget_list.remove(&variable_name);
            }

            let mut variable_to_insert_list = Vec::new();
            for (variable_name, variable_value) in &filter_config.variables {
                if !self.variable_widget_list.contains_key(variable_name) {
                    let (variable_wrapper, variable_input) =
                        variable::build_variable_row(&self.relm, variable_name, &variable_value);

                    self.model
                        .config
                        .variables
                        .insert(variable_name.clone(), variable_value.clone());

                    self.variable_list_container.add(&variable_wrapper);

                    variable_to_insert_list
                        .push((variable_name.clone(), (variable_wrapper, variable_input)));
                }
            }

            self.variable_widget_list.extend(variable_to_insert_list);
            self.variable_list_container.show_all();
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
        Relm<crate::Win>,
    );
    type Msg = RenderStageConfigViewMsg;

    fn model(
        _: &Relm<Self>,
        model: (
            Uuid,
            PathBuf,
            RenderStageConfig,
            Vec<String>,
            Relm<crate::Win>,
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
            RenderStageConfigViewMsg::SetName(new_name) => self.model.config.name = new_name,
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
            RenderStageConfigViewMsg::UpdateVariableList => self.update_variable_list(),
            RenderStageConfigViewMsg::UpdateInputChoiceList(choice_list) => {
                self.update_input_choice_list(&choice_list);
            }
        }

        self.model
            .parent_relm
            .stream()
            .emit(crate::Msg::UpdateRenderStageConfig(
                self.model.id,
                self.model.config.clone(),
            ));
    }
}

impl Widget for RenderStageConfigView {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let mut input_widget_list = HashMap::new();
        let mut variable_widget_list = HashMap::new();

        let root = gtk::Box::new(Vertical, 0);
        root.override_background_color(
            StateFlags::NORMAL,
            Some(&RGBA {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.125,
            }),
        );

        // Building of the input name row
        let name_row = gtk::Box::new(Horizontal, 8);
        name_row.set_property_margin(8);

        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);
        name_label.set_size_request(48, 0);

        let name_entry = Entry::new();
        name_entry.set_text(&model.config.name);
        name_entry.set_hexpand(true);
        connect!(
            relm,
            name_entry,
            connect_changed(val),
            Some(RenderStageConfigViewMsg::SetName(
                val.get_text().to_string()
            ))
        );

        name_row.add(&name_label);
        name_row.add(&name_entry);

        // Building of the filter selection row
        let filter_row = gtk::Box::new(Horizontal, 8);
        filter_row.set_property_margin(8);

        let filter_label = Label::new(Some("Filter: "));
        filter_label.set_xalign(0.0);
        filter_label.set_size_request(48, 0);

        let available_filters = load_available_filter_list(&model.project_path).unwrap();
        let filter_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in available_filters.keys() {
            filter_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        filter_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);

        let filter_chooser = gtk::ComboBoxText::new();
        filter_chooser.set_model(Some(&filter_store));
        filter_chooser.set_hexpand(true);

        filter_chooser.set_id_column(0);
        filter_chooser.set_entry_text_column(1);

        {
            let filter_chooser = filter_chooser.clone();
            connect!(
                relm,
                filter_chooser.clone(),
                connect_changed(_),
                Some(RenderStageConfigViewMsg::SetFilter(
                    filter_chooser.get_active_id().unwrap().to_string()
                ))
            );
        }

        filter_row.add(&filter_label);
        filter_row.add(&filter_chooser);

        // Building of the precision selection row
        let precision_row = gtk::Box::new(Horizontal, 8);
        precision_row.set_property_margin(8);

        let precision_label = Label::new(Some("Precision: "));
        precision_label.set_xalign(0.0);
        precision_label.set_size_request(48, 0);

        let available_precisions = ["U8", "F16", "F32"];
        let precision_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in available_precisions.iter() {
            precision_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        precision_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
        precision_store.set_default_sort_func(&list_store_sort_function);

        let precision_chooser = gtk::ComboBoxText::new();
        precision_chooser.set_model(Some(&precision_store));
        precision_chooser.set_hexpand(true);

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
                &precision_chooser.clone(),
                connect_changed(_),
                Some(RenderStageConfigViewMsg::SetPrecision(
                    precision_chooser.get_active_id().unwrap().to_string()
                ))
            );
        }

        precision_row.add(&precision_label);
        precision_row.add(&precision_chooser);

        // Building of a container for the inputs and variables configuration
        let tabs_container = Notebook::new();
        tabs_container.set_property_margin(8);

        // Begin of input list composition panel
        let input_tab = gtk::Box::new(Vertical, 0);
        input_tab.set_property_margin(0);

        let input_list_container = gtk::Box::new(Vertical, 8);

        input_tab.add(&input_list_container);

        // Begin of input list composition panel
        let variable_tab = gtk::Box::new(Vertical, 0);
        variable_tab.set_property_margin(0);

        let variable_list_container = gtk::Box::new(Vertical, 8);

        variable_tab.add(&variable_list_container);

        tabs_container.append_page(&input_tab, Some(&Label::new(Some("Inputs"))));
        tabs_container.append_page(&variable_tab, Some(&Label::new(Some("Variables"))));

        root.add(&name_row);
        root.add(&filter_row);
        root.add(&precision_row);
        root.add(&tabs_container);

        if available_filters.contains_key(&model.config.filter) {
            let filter = available_filters
                .get(&model.config.filter)
                .unwrap()
                .1
                .clone();

            filter_chooser.set_active_id(Some(&model.config.filter));

            for input_name in &filter.inputs {
                let input_value = if let Some(input_value) = model.config.inputs.get(input_name) {
                    input_value.clone()
                } else {
                    SampledInput::Linear("".to_string())
                };

                let (input_wrapper, input_type_entry, input_value_entry) = input::build_input_row(
                    relm,
                    &model.input_choice_list,
                    &input_name,
                    &input_value,
                );

                input_list_container.add(&input_wrapper);
                input_widget_list.insert(
                    input_name.clone(),
                    (input_wrapper, input_type_entry, input_value_entry),
                );
            }

            for (variable_name, variable_value) in &filter.variables {
                let variable_value =
                    if let Some(stage_variable_value) = model.config.variables.get(variable_name) {
                        stage_variable_value
                    } else {
                        variable_value
                    };
                let (variable_wrapper, variable_input) =
                    variable::build_variable_row(relm, &variable_name, &variable_value);

                variable_list_container.add(&variable_wrapper);
                variable_widget_list
                    .insert(variable_name.clone(), (variable_wrapper, variable_input));
            }
        }

        Self {
            relm: relm.clone(),
            model,
            root,

            filter_chooser,
            precision_chooser,

            input_list_container,
            input_widget_list,

            variable_list_container,
            variable_widget_list,
        }
    }
}

pub fn load_available_filter_list(
    project_path: &Path,
) -> Result<HashMap<String, (PathBuf, FilterConfig)>> {
    let mut available_filter_list = HashMap::new();

    let project_filter_folder_path = project_path.join("filters");
    let wvr_filter_folder_path = wvr_data::get_filters_path();

    // Load filters from project
    for folder_entry in project_filter_folder_path.read_dir()? {
        let filter_path = folder_entry?.path();
        let filter_config_path = filter_path.join("config.json");
        if !filter_config_path.exists() {
            continue;
        }

        let filter_name = filter_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let filter_config: FilterConfig =
            serde_json::from_reader::<File, FilterConfig>(File::open(&filter_config_path)?)
                .unwrap();

        available_filter_list.insert(filter_name, (filter_path, filter_config));
    }

    // Load filters provided by wvr
    for folder_entry in wvr_filter_folder_path.read_dir()? {
        let filter_path = folder_entry?.path();
        let filter_config_path = filter_path.join("config.json");
        if !filter_config_path.exists() {
            continue;
        }

        let filter_name = filter_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let filter_config: FilterConfig =
            serde_json::from_reader::<File, FilterConfig>(File::open(&filter_config_path)?)
                .unwrap();

        available_filter_list
            .entry(filter_name)
            .or_insert((filter_path, filter_config));
    }

    Ok(available_filter_list)
}

pub fn list_store_sort_function(
    model: &TreeModel,
    iter_a: &TreeIter,
    iter_b: &TreeIter,
) -> Ordering {
    model
        .get_value(&iter_a, 0)
        .get::<String>()
        .unwrap()
        .unwrap()
        .cmp(
            &model
                .get_value(&iter_b, 0)
                .get::<String>()
                .unwrap()
                .unwrap(),
        )
}
