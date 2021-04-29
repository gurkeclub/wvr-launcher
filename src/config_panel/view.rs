use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use uuid::Uuid;

use anyhow::Result;

use glib::Cast;

use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    prelude::{GtkListStoreExtManual, NotebookExtManual, TreeSortableExtManual},
    BoxExt,
};
use gtk::{
    AspectFrame, Button, ButtonExt, ComboBoxExt, ComboBoxText, ContainerExt, FrameExt, GLArea,
    GLAreaExt, GtkListStoreExt, Label, LabelExt, Notebook, NotebookExt, Paned, PanedExt,
    ReliefStyle, ShadowType, SortColumn, SortType, WidgetExt,
};

use relm::{connect, Component, Relm, Update, Widget};

use path_calculate::Calculate;
use strsim::levenshtein;

use wvr::utils::load_available_filter_list;
use wvr_com::data::{Message, RenderStageUpdate};
use wvr_data::config::project_config::{Automation, InputConfig, ProjectConfig, SampledInput};

use crate::input_config;
use crate::server_config;
use crate::stage_config;
use crate::view_config;

use crate::input_config::InputConfigViewMsg;
use crate::stage_config::view::{RenderStageConfigView, RenderStageConfigViewMsg};

use super::get_input_choice_list;
use super::msg::ConfigPanelMsg;

pub struct Model {
    parent_relm: Relm<crate::main_window::MainWindow>,
    project_path: PathBuf,
    config: ProjectConfig,
    control_channel: Option<Sender<Message>>,
}

pub struct ConfigPanel {
    model: Model,

    project_container: Paned,

    config_container: gtk::Box,

    input_list_container: gtk::Box,
    input_config_widget_list: HashMap<Uuid, (String, InputConfig, gtk::Box)>,

    created_render_stage_count: usize,
    render_stage_config_list_container: Notebook,
    render_stage_config_widget_list: HashMap<Uuid, (Component<RenderStageConfigView>, gtk::Box)>,
    render_stage_order: Vec<Uuid>,

    final_stage_name_chooser: ComboBoxText,

    control_container: gtk::Box,

    glarea: GLArea,

    relm: Relm<Self>,
}

impl ConfigPanel {
    pub fn get_render_stage_index(&self, render_stage_id: &Uuid) -> Option<usize> {
        self.render_stage_order
            .iter()
            .enumerate()
            .find(|(_, candidate)| candidate == &render_stage_id)
            .map(|(index, _)| index)
    }

    pub fn get_input_name(&self, input_id: &Uuid) -> Option<String> {
        self.input_config_widget_list
            .get(input_id)
            .map(|(input_name, _, _)| input_name.clone())
    }

    fn save_config(&mut self, project_config_file_path: &Path) {
        println!("Saving to {:?}", project_config_file_path);

        let config_path = project_config_file_path;
        if let Ok(mut project_config_file) = std::fs::File::create(config_path) {
            let config_as_bytes = serde_json::ser::to_string_pretty(&self.model.config)
                .unwrap()
                .into_bytes();

            project_config_file.write_all(&config_as_bytes).unwrap();
        }
    }

    fn start_wvr(&mut self) -> Result<()> {
        if self.model.control_channel.is_none() {
            let order_sender = crate::wvr_frame::build_wvr_frame(
                &self.glarea,
                &self.model.project_path,
                &self.model.config,
            )?;

            self.model.control_channel = Some(order_sender);
        }

        Ok(())
    }
}

impl Update for ConfigPanel {
    type Model = Model;
    type ModelParam = (Relm<crate::main_window::MainWindow>, PathBuf, ProjectConfig);
    type Msg = ConfigPanelMsg;

    fn model(
        _: &Relm<Self>,
        project: (Relm<crate::main_window::MainWindow>, PathBuf, ProjectConfig),
    ) -> Self::Model {
        Model {
            parent_relm: project.0,
            project_path: project.1,
            config: project.2,
            control_channel: None,
        }
    }

    fn update(&mut self, event: ConfigPanelMsg) {
        let mut render_stage_update_message_list = Vec::new();

        if let Some(message) = event.to_wvr_message(self) {
            render_stage_update_message_list.push(message);
        }

        let mut input_list_changed = false;

        let input_choice_list = get_input_choice_list(&self.model.config);

        match &event {
            ConfigPanelMsg::StartProject => {
                self.start_wvr().unwrap();
            }
            ConfigPanelMsg::PauseProject => (),
            ConfigPanelMsg::StopProject => (),
            ConfigPanelMsg::SetBpm(bpm) => {
                self.model.config.bpm = *bpm as f32;
            }
            ConfigPanelMsg::SetWidth(width) => {
                self.model.config.view.width = *width;
            }
            ConfigPanelMsg::SetHeight(height) => {
                self.model.config.view.height = *height;
            }
            ConfigPanelMsg::SetTargetFps(fps) => {
                self.model.config.view.target_fps = *fps as f32;
            }
            ConfigPanelMsg::SetDynamicResolution(dynamic) => {
                self.model.config.view.dynamic = *dynamic;
            }
            ConfigPanelMsg::SetVSync(vsync) => {
                self.model.config.view.vsync = *vsync;
            }
            ConfigPanelMsg::SetScreenshot(screenshot) => {
                self.model.config.view.screenshot = *screenshot;
            }
            ConfigPanelMsg::SetFullscreen(fullscreen) => {
                self.model.config.view.fullscreen = *fullscreen;
            }
            ConfigPanelMsg::SetLockedSpeed(locked_speed) => {
                self.model.config.view.locked_speed = *locked_speed;
            }

            ConfigPanelMsg::SetServerIp(ip) => self.model.config.server.ip = ip.clone(),
            ConfigPanelMsg::SetServerPort(port) => self.model.config.server.port = *port as usize,
            ConfigPanelMsg::SetServerEnabled(enable) => self.model.config.server.enable = *enable,

            ConfigPanelMsg::Save => {
                self.save_config(&self.model.project_path.join("config.json"));
            }

            ConfigPanelMsg::AddInput(input_name, input_config) => {
                let (id, wrapper) = input_config::build_input_config_row(
                    &self.relm,
                    &self.model.project_path,
                    &input_name,
                    &input_config,
                );

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list
                    .insert(id, (input_name.clone(), input_config.clone(), wrapper));

                input_list_changed = true;
            }
            ConfigPanelMsg::RemoveInput(id) => {
                if let Some((_, _, input_view_wrapper)) = self.input_config_widget_list.get(&id) {
                    self.input_list_container.remove(input_view_wrapper);
                }
                self.input_config_widget_list.remove(&id);

                input_list_changed = true;
            }
            ConfigPanelMsg::UpdateInput(id, input_update_message) => {
                if let Some((ref mut input_name, ref mut config, _)) =
                    self.input_config_widget_list.get_mut(&id)
                {
                    if let InputConfigViewMsg::SetName(new_input_name) = &input_update_message {
                        if self.model.config.inputs.contains_key(new_input_name) {
                            return;
                        }

                        let config = self.model.config.inputs.remove(input_name).unwrap();
                        self.model
                            .config
                            .inputs
                            .insert(new_input_name.clone(), config);

                        *input_name = new_input_name.clone();

                        input_list_changed = true;
                    } else {
                        match config {
                            InputConfig::Cam {
                                path,
                                width,
                                height,
                            } => match &input_update_message {
                                InputConfigViewMsg::SetPath(new_path) => *path = new_path.clone(),
                                InputConfigViewMsg::SetHeight(new_height) => {
                                    *height = *new_height as usize
                                }
                                InputConfigViewMsg::SetWidth(new_width) => {
                                    *width = *new_width as usize
                                }
                                _ => unreachable!(),
                            },
                            InputConfig::Video {
                                path,
                                width,
                                height,
                                speed,
                            } => match &input_update_message {
                                InputConfigViewMsg::SetPath(new_path) => {
                                    if let Ok(new_path) =
                                        PathBuf::from(new_path).related_to(&self.model.project_path)
                                    {
                                        *path = new_path.to_str().unwrap().to_string();
                                    }
                                }
                                InputConfigViewMsg::SetHeight(new_height) => {
                                    *height = *new_height as usize
                                }
                                InputConfigViewMsg::SetWidth(new_width) => {
                                    *width = *new_width as usize
                                }
                                InputConfigViewMsg::SetSpeed(new_speed) => *speed = *new_speed,

                                _ => unreachable!(),
                            },

                            InputConfig::Picture {
                                path,
                                width,
                                height,
                            } => match &input_update_message {
                                InputConfigViewMsg::SetPath(new_path) => {
                                    if let Ok(new_path) =
                                        PathBuf::from(new_path).related_to(&self.model.project_path)
                                    {
                                        *path = new_path.to_str().unwrap().to_string();
                                    }
                                }
                                InputConfigViewMsg::SetHeight(new_height) => {
                                    *height = *new_height as usize
                                }
                                InputConfigViewMsg::SetWidth(new_width) => {
                                    *width = *new_width as usize
                                }
                                _ => unreachable!(),
                            },
                            InputConfig::Midi { name } => match &input_update_message {
                                InputConfigViewMsg::SetPath(new_path) => *name = new_path.clone(),
                                _ => unreachable!(),
                            },
                        }
                        self.model
                            .config
                            .inputs
                            .insert(input_name.to_owned(), config.clone());
                    }
                }
            }

            ConfigPanelMsg::AddRenderStage(render_stage_config) => {
                self.created_render_stage_count += 1;

                let mut available_filter_list =
                    load_available_filter_list(&wvr_data::get_filters_path(), true).unwrap();
                available_filter_list.extend(
                    load_available_filter_list(&self.model.project_path.join("filters"), false)
                        .unwrap(),
                );

                let (id, wrapper, render_stage_config_view) =
                    stage_config::build_render_stage_config_row(
                        &self.relm,
                        render_stage_config,
                        &input_choice_list,
                        &available_filter_list,
                    );

                let page_label_container = gtk::Box::new(Horizontal, 4);

                let page_label = Label::new(Some(&render_stage_config.name));
                page_label.set_xalign(0.0);
                page_label.set_hexpand(true);

                let remove_button = Button::new();
                remove_button.set_relief(ReliefStyle::None);
                remove_button.set_label("x");
                {
                    connect!(
                        self.relm,
                        remove_button,
                        connect_clicked(_),
                        Some(ConfigPanelMsg::RemoveRenderStage(id))
                    );
                }

                page_label_container.add(&remove_button);
                page_label_container.add(&page_label);

                self.render_stage_config_list_container
                    .append_page(&wrapper, Some(&page_label_container));
                self.render_stage_config_list_container
                    .set_tab_reorderable(&wrapper, true);

                page_label_container.show_all();
                wrapper.show_all();

                self.model
                    .config
                    .render_chain
                    .push(render_stage_config.clone());

                self.render_stage_order.push(id);

                self.render_stage_config_widget_list
                    .insert(id, (render_stage_config_view, wrapper));

                self.render_stage_config_list_container
                    .set_current_page(Some(
                        self.render_stage_config_list_container.get_n_pages() - 1,
                    ))
            }

            ConfigPanelMsg::RemoveRenderStage(id) => {
                if let Some((_, render_stage_config_view_wrapper)) =
                    self.render_stage_config_widget_list.get(&id)
                {
                    if let Some(render_stage_index) = self.get_render_stage_index(id) {
                        self.model.config.render_chain.remove(render_stage_index);
                        self.render_stage_config_list_container
                            .remove(render_stage_config_view_wrapper);
                    }
                }

                self.render_stage_config_widget_list.remove(&id);
                self.render_stage_order.remove(
                    self.render_stage_order
                        .iter()
                        .enumerate()
                        .find(|(_, candidate)| candidate == &id)
                        .unwrap()
                        .0,
                );
            }
            ConfigPanelMsg::UpdateRenderStageVariable(id, variable_name, variable_value) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        if let Some((old_variable_value, _)) =
                            config.variables.get_mut(variable_name)
                        {
                            *old_variable_value = variable_value.clone();
                        } else {
                            config.variables.insert(
                                variable_name.clone(),
                                (variable_value.clone(), Automation::None),
                            );
                        }
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStageVariableAutomation(
                id,
                variable_name,
                variable_automation,
            ) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        if let Some((_, old_variable_automation)) =
                            config.variables.get_mut(variable_name)
                        {
                            *old_variable_automation = *variable_automation;
                        }
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStageInput(id, input_name, new_input_value) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        config
                            .inputs
                            .insert(input_name.clone(), new_input_value.clone());
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStageFilterModeParams(id, new_filter_mode_params) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        config.filter_mode_params = *new_filter_mode_params;
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStagePrecision(id, new_precision) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        config.precision = *new_precision;
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStageName(id, new_name) => {
                if let Some((_, render_stage_config_view_wrapper)) =
                    self.render_stage_config_widget_list.get(&id)
                {
                    if let Ok(page_label_container) = self
                        .render_stage_config_list_container
                        .get_tab_label(render_stage_config_view_wrapper)
                        .unwrap()
                        .downcast::<gtk::Box>()
                    {
                        if let Some(page_label) = page_label_container.get_children().get(1) {
                            if let Some(page_label) = page_label.downcast_ref::<Label>() {
                                page_label.set_text(new_name.as_str());
                            }
                        }
                    }

                    if let Some(render_stage_index) = self.get_render_stage_index(id) {
                        if let Some(ref mut config) =
                            self.model.config.render_chain.get_mut(render_stage_index)
                        {
                            if &config.name != new_name {
                                config.name = new_name.clone();
                                input_list_changed = true;
                            }
                        }
                    }
                }
            }
            ConfigPanelMsg::UpdateRenderStageFilter(id, new_filter) => {
                if let Some(render_stage_index) = self.get_render_stage_index(id) {
                    if let Some(ref mut config) =
                        self.model.config.render_chain.get_mut(render_stage_index)
                    {
                        config.filter = new_filter.clone();
                    }
                }
            }
            ConfigPanelMsg::MoveStage(stage_id, target_index) => {
                let original_index = self.get_render_stage_index(stage_id).unwrap();

                let render_stage_config = self.model.config.render_chain.remove(original_index);
                self.model
                    .config
                    .render_chain
                    .insert(*target_index, render_stage_config);

                self.render_stage_order.remove(original_index);
                self.render_stage_order.insert(*target_index, *stage_id);
            }

            ConfigPanelMsg::UpdateRenderedTextureName(input) => {
                self.model
                    .config
                    .final_stage
                    .inputs
                    .insert("iChannel0".to_string(), input.clone());
            }
        }

        if input_list_changed {
            self.model.config.inputs.clear();
            for (name, config, _) in self.input_config_widget_list.values() {
                self.model
                    .config
                    .inputs
                    .insert(name.clone(), config.clone());
            }
        }

        let new_input_choice_list = get_input_choice_list(&self.model.config);
        if new_input_choice_list != input_choice_list {
            for (render_stage_config_widget, _) in self.render_stage_config_widget_list.values() {
                render_stage_config_widget.emit(RenderStageConfigViewMsg::UpdateInputChoiceList(
                    new_input_choice_list.clone(),
                ));
            }

            let current_id = if let Some(id) = self.final_stage_name_chooser.get_active_id() {
                id.to_string()
            } else {
                new_input_choice_list[0].clone()
            };

            // Update input choice for rendered texture chooser
            match new_input_choice_list.get(0) {
                Some(mut closest_id) => {
                    let mut closest_id_distance = levenshtein(&current_id, &closest_id);

                    let input_name_store = self
                        .final_stage_name_chooser
                        .get_model()
                        .unwrap()
                        .downcast::<gtk::ListStore>()
                        .unwrap();
                    input_name_store.clear();

                    for name in &new_input_choice_list {
                        input_name_store.insert_with_values(None, &[0, 1], &[name, name]);

                        let candidate_id_distance = levenshtein(&current_id, &name);

                        if candidate_id_distance < closest_id_distance {
                            closest_id = name;
                            closest_id_distance = candidate_id_distance;
                        }
                    }

                    let new_final_stage_input = SampledInput::Mipmaps(closest_id.clone());

                    self.model
                        .config
                        .final_stage
                        .inputs
                        .insert("iChannel0".to_string(), new_final_stage_input.clone());
                    self.final_stage_name_chooser
                        .set_active_id(Some(&closest_id));

                    render_stage_update_message_list.push(Message::UpdateFinalStage(
                        RenderStageUpdate::Input("iChannel0".to_string(), new_final_stage_input),
                    ));
                }
                None => {
                    self.final_stage_name_chooser.set_active_id(None);
                }
            }

            for (render_stage_index, render_stage_config) in
                self.model.config.render_chain.iter().enumerate()
            {
                for (input_name, input) in &render_stage_config.inputs {
                    render_stage_update_message_list.push(Message::UpdateRenderStage(
                        render_stage_index,
                        RenderStageUpdate::Input(input_name.clone(), input.clone()),
                    ));
                }
            }
        }

        if let Some(control_channel) = &mut self.model.control_channel {
            for message in render_stage_update_message_list {
                control_channel.send(message).unwrap();
            }
        }

        self.model
            .parent_relm
            .stream()
            .emit(crate::main_window::Msg::UpdateConfig(
                self.model.config.clone(),
            ));
    }
}

impl Widget for ConfigPanel {
    type Root = Paned;

    fn root(&self) -> Self::Root {
        self.project_container.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let mut input_config_widget_list = HashMap::new();
        let mut render_stage_config_widget_list = HashMap::new();

        let model = model;

        let project_container = Paned::new(Horizontal);

        let misc_view_container = Paned::new(Vertical);
        let config_container = gtk::Box::new(Vertical, 0);

        let tabs_container = Notebook::new();
        tabs_container.set_tab_pos(gtk::PositionType::Top);
        tabs_container.set_show_border(false);

        let view_config_widget =
            view_config::build_view(relm, model.config.bpm as f64, &model.config.view);

        let server_config_panel = server_config::build_view(relm, &model.config.server);

        let (input_list_panel, input_list_container) = input_config::build_list_view(
            relm,
            &model.project_path,
            &mut input_config_widget_list,
            &model.config.inputs,
        );

        let render_stage_panel = gtk::Box::new(Vertical, 4);
        render_stage_panel.set_vexpand(true);

        let (render_stage_config_list_container, render_stage_order) =
            stage_config::build_list_view(
                relm,
                &model.project_path,
                &model.config.render_chain,
                &get_input_choice_list(&model.config),
                &mut render_stage_config_widget_list,
            );

        render_stage_panel.add(&render_stage_config_list_container);

        //tabs_container.append_page(&view_config_widget, Some(&Label::new(Some("General"))));
        tabs_container.append_page(&server_config_panel, Some(&Label::new(Some("Server"))));
        tabs_container.append_page(&input_list_panel, Some(&Label::new(Some("Inputs"))));
        tabs_container.append_page(&render_stage_panel, Some(&Label::new(Some("Layers"))));

        /*
        tabs_container
            .get_tab_label(&view_config_widget)
            .unwrap()
            .set_tooltip_text(Some("Configure general parameters."));
            */
        tabs_container
            .get_tab_label(&server_config_panel)
            .unwrap()
            .set_tooltip_text(Some(
                "/!\\ Not Implemented /!\\ \nConfigure wvr control server.",
            ));
        tabs_container
            .get_tab_label(&input_list_panel)
            .unwrap()
            .set_tooltip_text(Some("Configure inputs."));
        tabs_container
            .get_tab_label(&render_stage_panel)
            .unwrap()
            .set_tooltip_text(Some("Configure layers."));

        config_container.add(&tabs_container);

        let view_container = gtk::Box::new(Vertical, 4);
        view_container.set_hexpand(true);
        view_container.set_vexpand(true);
        view_container.set_property_margin(8);

        let control_container = gtk::Box::new(Horizontal, 2);

        let pause_button = Button::new();
        pause_button.set_label("Pause");
        pause_button.set_hexpand(true);
        connect!(
            relm,
            pause_button,
            connect_clicked(_),
            Some(ConfigPanelMsg::PauseProject)
        );

        let start_button = Button::new();
        start_button.set_label("Start");
        start_button.set_hexpand(true);
        connect!(
            relm,
            start_button,
            connect_clicked(_),
            Some(ConfigPanelMsg::StartProject)
        );

        // Building the row allowing selection of the texture to render
        let final_stage_row = gtk::Box::new(Horizontal, 8);

        let final_stage_label = Label::new(Some("Displayed layer:"));

        let input_name_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in &get_input_choice_list(&model.config) {
            input_name_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        input_name_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
        input_name_store.set_default_sort_func(&stage_config::list_store_sort_function);

        let final_stage_name_chooser = gtk::ComboBoxText::new();
        final_stage_name_chooser.set_hexpand(true);
        final_stage_name_chooser.set_model(Some(&input_name_store));

        final_stage_name_chooser.set_id_column(0);
        final_stage_name_chooser.set_entry_text_column(1);

        match model.config.final_stage.inputs.values().next().unwrap() {
            SampledInput::Linear(input_name) => {
                final_stage_name_chooser.set_active_id(Some(input_name));
            }

            SampledInput::Nearest(input_name) => {
                final_stage_name_chooser.set_active_id(Some(input_name));
            }
            SampledInput::Mipmaps(input_name) => {
                final_stage_name_chooser.set_active_id(Some(input_name));
            }
        }

        {
            let final_stage_name_chooser = final_stage_name_chooser.clone();
            connect!(
                relm,
                final_stage_name_chooser,
                connect_changed(chooser),
                Some(ConfigPanelMsg::UpdateRenderedTextureName(
                    SampledInput::Mipmaps(
                        chooser
                            .get_active_id()
                            .unwrap_or_else(|| glib::GString::from(""))
                            .to_string(),
                    )
                ))
            );
        }

        //final_stage_row.add(&final_stage_label);
        //final_stage_row.add(&final_stage_name_chooser);

        control_container.add(&final_stage_name_chooser);
        control_container.add(&pause_button);
        control_container.add(&start_button);

        let glarea_wrapper = AspectFrame::new(None, 0.5, 0.5, 16.0 / 9.0, true);
        glarea_wrapper.set_shadow_type(ShadowType::None);
        glarea_wrapper.set_hexpand(true);
        glarea_wrapper.set_vexpand(true);
        glarea_wrapper.set_label_align(0.5, 0.0);
        glarea_wrapper.set_label_widget(Some(&control_container));

        let glarea = GLArea::new();

        glarea.set_size_request(
            model.config.view.width as i32 / 4,
            model.config.view.height as i32 / 4,
        );

        glarea.set_required_version(3, 2);
        glarea.set_hexpand(true);
        glarea.set_vexpand(true);

        glarea_wrapper.add(&glarea);

        view_container.add(&glarea_wrapper);
        //view_container.add(&control_container);
        //view_container.add(&final_stage_row);

        misc_view_container.pack1(&view_container, true, false);
        misc_view_container.pack2(&view_config_widget, true, false);

        project_container.pack1(&config_container, true, false);
        project_container.pack2(&misc_view_container, true, false);

        project_container.show_all();

        let created_render_stage_count = model.config.render_chain.len();

        Self {
            model,

            project_container,

            config_container,

            input_list_container,

            input_config_widget_list,

            created_render_stage_count,
            render_stage_config_list_container,
            render_stage_config_widget_list,
            render_stage_order,

            final_stage_name_chooser,

            glarea,
            control_container,

            relm: relm.clone(),
        }
    }
}
