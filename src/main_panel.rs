use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use uuid::Uuid;

use glib::object::ObjectExt;
use glib::Cast;

use gtk::prelude::{GtkListStoreExtManual, NotebookExtManual, TreeSortableExtManual};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Button, ButtonExt, ComboBoxExt, ComboBoxText, ContainerExt, GtkListStoreExt, Label, LabelExt,
    Notebook, NotebookExt, Settings, SortColumn, SortType, WidgetExt,
};

use relm::{connect, Component, Relm, Update, Widget};

use strsim::levenshtein;

use wvr_data::config::project_config::ProjectConfig;
use wvr_data::config::project_config::{
    BufferPrecision, FilterMode, InputConfig, RenderStageConfig, SampledInput, Speed,
};

use crate::input_config;
use crate::server_config;
use crate::stage_config;
use crate::view_config;

use crate::input_config::InputConfigView;
use crate::stage_config::RenderStageConfigView;
use crate::stage_config::RenderStageConfigViewMsg;
use crate::Msg;

pub fn get_input_choice_list(config: &ProjectConfig) -> Vec<String> {
    let mut result: Vec<String> = config
        .inputs
        .keys()
        .map(String::clone)
        .chain(config.render_chain.iter().map(|stage| stage.name.clone()))
        .collect();

    result.sort();

    result
}

pub struct Model {
    project_path: PathBuf,
    config: ProjectConfig,
}

pub struct MainPanel {
    model: Model,

    window_container: gtk::Box,

    input_list_container: gtk::Box,
    input_config_widget_list:
        HashMap<Uuid, (String, InputConfig, Component<InputConfigView>, gtk::Box)>,

    render_stage_config_list_container: Notebook,
    render_stage_config_widget_list: HashMap<
        Uuid,
        (
            RenderStageConfig,
            Component<RenderStageConfigView>,
            gtk::Box,
        ),
    >,
    render_stage_order: Vec<Uuid>,

    renderered_stage_name_chooser: ComboBoxText,

    relm: Relm<Self>,
}

impl MainPanel {
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

    fn start(&mut self) {
        let config_path = self.model.project_path.join("config.tmp.json");

        self.save_config(&config_path);

        //self.window.hide();

        Command::new("wvr")
            .arg("-c")
            .arg(config_path.to_str().unwrap())
            .output()
            .expect("failed to execute process");

        //self.window.show();
    }
}

impl Update for MainPanel {
    type Model = Model;
    type ModelParam = (PathBuf, ProjectConfig);
    type Msg = Msg;

    fn model(_: &Relm<Self>, project: (PathBuf, ProjectConfig)) -> Self::Model {
        Model {
            project_path: project.0,
            config: project.1,
        }
    }

    fn update(&mut self, event: Msg) {
        let mut input_list_changed = false;
        let mut render_chain_changed = false;

        let input_choice_list = get_input_choice_list(&self.model.config);

        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::SetBpm(bpm) => self.model.config.bpm = bpm as f32,
            Msg::SetWidth(width) => self.model.config.view.width = width,
            Msg::SetHeight(height) => self.model.config.view.height = height,
            Msg::SetTargetFps(fps) => self.model.config.view.target_fps = fps as f32,
            Msg::SetDynamicResolution(dynamic) => self.model.config.view.dynamic = dynamic,
            Msg::SetVSync(vsync) => self.model.config.view.vsync = vsync,
            Msg::SetScreenshot(screenshot) => self.model.config.view.screenshot = screenshot,
            Msg::SetFullscreen(fullscreen) => self.model.config.view.fullscreen = fullscreen,
            Msg::SetLockedSpeed(locked_speed) => self.model.config.view.locked_speed = locked_speed,

            Msg::SetServerIp(ip) => self.model.config.server.ip = ip,
            Msg::SetServerPort(port) => self.model.config.server.port = port as usize,
            Msg::SetServerEnabled(enable) => self.model.config.server.enable = enable,

            Msg::Save => {
                self.save_config(&self.model.project_path.join("config.json"));
            }
            Msg::Start => {
                self.start();
            }
            Msg::Error(e) => eprintln!("{:?}", e),

            Msg::AddCamInput => {
                let input_cam_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_cam())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Camera #{:}", input_cam_count + 1);

                let input_config = InputConfig::Cam {
                    path: "/dev/video0".to_string(),
                    width: 640,
                    height: 480,
                };

                let (id, wrapper, input_config_view) = input_config::build_input_config_row(
                    &self.relm,
                    &self.model.project_path,
                    &input_name,
                    &input_config,
                );

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list
                    .insert(id, (input_name, input_config, input_config_view, wrapper));

                input_list_changed = true;
            }
            Msg::AddVideoInput => {
                let input_video_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_video())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Video #{:}", input_video_count + 1);

                let input_config = InputConfig::Video {
                    path: "res/example_video.mp4".to_string(),
                    width: 640,
                    height: 480,
                    speed: Speed::Fps(25.0),
                };

                let (id, wrapper, input_config_view) = input_config::build_input_config_row(
                    &self.relm,
                    &self.model.project_path,
                    &input_name,
                    &input_config,
                );

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list
                    .insert(id, (input_name, input_config, input_config_view, wrapper));

                input_list_changed = true;
            }
            Msg::AddPictureInput => {
                let input_picture_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_picture())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Picture #{:}", input_picture_count + 1);

                let input_config = InputConfig::Picture {
                    path: "res/example_picture.png".to_string(),
                    width: 640,
                    height: 480,
                };

                let (id, wrapper, input_config_view) = input_config::build_input_config_row(
                    &self.relm,
                    &self.model.project_path,
                    &input_name,
                    &input_config,
                );

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list
                    .insert(id, (input_name, input_config, input_config_view, wrapper));

                input_list_changed = true;
            }
            Msg::AddMidiInput => {
                let input_midi_count = self
                    .input_config_widget_list
                    .values()
                    .filter(|(_, input_config, _, _)| input_config.is_midi())
                    .map(|_| 1)
                    .sum::<usize>();

                let input_name = format!("Midi #{:}", input_midi_count + 1);

                let input_config = InputConfig::Midi {
                    name: "*".to_string(),
                };

                let (id, wrapper, input_config_view) = input_config::build_input_config_row(
                    &self.relm,
                    &self.model.project_path,
                    &input_name,
                    &input_config,
                );

                self.input_list_container.add(&wrapper);
                wrapper.show_all();
                self.input_config_widget_list
                    .insert(id, (input_name, input_config, input_config_view, wrapper));

                input_list_changed = true;
            }
            Msg::RemoveInput(id) => {
                if let Some((_, _, _, input_view_wrapper)) = self.input_config_widget_list.get(&id)
                {
                    self.input_list_container.remove(input_view_wrapper);
                }
                self.input_config_widget_list.remove(&id);

                input_list_changed = true;
            }
            Msg::UpdateInputConfig(id, new_name, new_config) => {
                if let Some((ref mut name, ref mut config, _, _)) =
                    self.input_config_widget_list.get_mut(&id)
                {
                    *name = new_name;
                    *config = new_config;
                }

                input_list_changed = true;
            }

            Msg::AddRenderStage => {
                let render_stage_name = format!(
                    "Layer #{:}",
                    self.render_stage_config_list_container.get_children().len()
                );
                let filter_name = "copy_image";

                let render_stage_config = RenderStageConfig {
                    name: render_stage_name,
                    filter: filter_name.to_string(),
                    filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 0.0, 0.0),
                    inputs: HashMap::new(),
                    variables: HashMap::new(),
                    precision: BufferPrecision::U8,
                };

                let (id, wrapper, render_stage_config_view) =
                    stage_config::build_render_stage_config_row(
                        &self.relm,
                        &self.model.project_path,
                        &render_stage_config,
                        &input_choice_list,
                    );

                let page_label_container = gtk::Box::new(Horizontal, 4);

                let page_label = Label::new(Some(&render_stage_config.name));
                page_label.set_xalign(0.0);
                page_label.set_hexpand(true);

                let remove_button = Button::new();
                remove_button.set_label("X");
                {
                    connect!(
                        self.relm,
                        remove_button,
                        connect_clicked(_),
                        Some(crate::Msg::RemoveRenderStage(id))
                    );
                }

                page_label_container.add(&remove_button);
                page_label_container.add(&page_label);

                self.render_stage_config_list_container
                    .append_page(&wrapper, Some(&page_label_container));

                page_label_container.show_all();
                wrapper.show_all();
                self.render_stage_config_widget_list
                    .insert(id, (render_stage_config, render_stage_config_view, wrapper));
                self.render_stage_order.push(id);

                render_chain_changed = true;
            }

            Msg::RemoveRenderStage(id) => {
                if let Some((_, _, render_stage_config_view_wrapper)) =
                    self.render_stage_config_widget_list.get(&id)
                {
                    self.render_stage_config_list_container
                        .remove(render_stage_config_view_wrapper);
                }
                self.render_stage_config_widget_list.remove(&id);
                if let Some(id_index) = self.render_stage_order.iter().position(|&n| n == id) {
                    self.render_stage_order.remove(id_index);
                }

                render_chain_changed = true;
            }
            Msg::UpdateRenderStageConfig(id, new_config) => {
                if let Some((ref mut config, _, render_stage_config_view_wrapper)) =
                    self.render_stage_config_widget_list.get_mut(&id)
                {
                    if let Ok(page_label_container) = self
                        .render_stage_config_list_container
                        .get_tab_label(render_stage_config_view_wrapper)
                        .unwrap()
                        .downcast::<gtk::Box>()
                    {
                        if let Some(page_label) = page_label_container.get_children().get(1) {
                            if let Some(page_label) = page_label.downcast_ref::<Label>() {
                                page_label.set_text(new_config.name.as_str());
                            }
                        }
                    }

                    *config = new_config;
                }

                render_chain_changed = true;
            }

            Msg::UpdateRenderedTextureName | Msg::UpdateRenderedTextureSampling => {
                self.model.config.final_stage.inputs.insert(
                    "iChannel0".to_string(),
                    SampledInput::Linear(
                        self.renderered_stage_name_chooser
                            .get_active_id()
                            .unwrap_or_else(|| glib::GString::from(""))
                            .to_string(),
                    ),
                );
            }
        }

        if input_list_changed {
            self.model.config.inputs.clear();
            for (name, config, _, _) in self.input_config_widget_list.values() {
                self.model
                    .config
                    .inputs
                    .insert(name.clone(), config.clone());
            }
        }

        if render_chain_changed {
            self.model.config.render_chain.clear();
            for id in &self.render_stage_order {
                let (config, _, _) = self.render_stage_config_widget_list.get(id).unwrap();
                self.model.config.render_chain.push(config.clone());
            }
        }

        let new_input_choice_list = get_input_choice_list(&self.model.config);
        if new_input_choice_list != input_choice_list {
            for (_, render_stage_config_widget, _) in self.render_stage_config_widget_list.values()
            {
                render_stage_config_widget.emit(RenderStageConfigViewMsg::UpdateInputChoiceList(
                    new_input_choice_list.clone(),
                ));
            }

            let current_id = if let Some(id) = self.renderered_stage_name_chooser.get_active_id() {
                id.to_string()
            } else {
                new_input_choice_list[0].clone()
            };

            // Update input choice for rendered texture chooser
            if let Some(mut closest_id) = new_input_choice_list.get(0) {
                let mut closest_id_distance = levenshtein(&current_id, &closest_id);

                let input_name_store = self
                    .renderered_stage_name_chooser
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

                if closest_id != &current_id {
                    self.renderered_stage_name_chooser
                        .set_active_id(Some(&closest_id));
                }
            } else {
                self.renderered_stage_name_chooser.set_active_id(None);
            }
        }
    }
}

impl Widget for MainPanel {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.window_container.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let settings = Settings::get_default().unwrap();
        settings
            .set_property("gtk-application-prefer-dark-theme", &true)
            .unwrap();

        let mut input_config_widget_list = HashMap::new();
        let mut render_stage_config_widget_list = HashMap::new();
        let mut render_stage_order = Vec::new();

        let model = model;

        let window_container = gtk::Box::new(Vertical, 0);

        let tabs_container = Notebook::new();
        tabs_container.set_tab_pos(gtk::PositionType::Left);
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

        let render_stage_config_list_container = stage_config::build_list_view(
            relm,
            &model.project_path,
            &model.config.render_chain,
            &get_input_choice_list(&model.config),
            &mut render_stage_config_widget_list,
            &mut render_stage_order,
        );

        render_stage_panel.add(&render_stage_config_list_container);

        tabs_container.append_page(&view_config_widget, Some(&Label::new(Some("General"))));
        tabs_container.append_page(&server_config_panel, Some(&Label::new(Some("Server"))));
        tabs_container.append_page(&input_list_panel, Some(&Label::new(Some("Inputs"))));
        tabs_container.append_page(&render_stage_panel, Some(&Label::new(Some("Layers"))));

        tabs_container
            .get_tab_label(&view_config_widget)
            .unwrap()
            .set_tooltip_text(Some("Configure general parameters."));
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
            .set_tooltip_text(Some("Configure the rendered layers."));

        let control_container = gtk::Box::new(Vertical, 8);
        control_container.set_property_margin(8);

        // Building the row allowing selection of the texture to render
        let final_stage_row = gtk::Box::new(Horizontal, 8);
        //final_stage_row.set_property_margin(8);

        let final_stage_label = Label::new(Some("Displayed layer:"));

        let input_name_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
        for name in &get_input_choice_list(&model.config) {
            input_name_store.insert_with_values(None, &[0, 1], &[name, name]);
        }
        input_name_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
        input_name_store.set_default_sort_func(&stage_config::list_store_sort_function);

        let renderered_stage_name_chooser = gtk::ComboBoxText::new();
        renderered_stage_name_chooser.set_hexpand(true);
        renderered_stage_name_chooser.set_model(Some(&input_name_store));

        renderered_stage_name_chooser.set_id_column(0);
        renderered_stage_name_chooser.set_entry_text_column(1);

        match model.config.final_stage.inputs.values().next().unwrap() {
            SampledInput::Linear(input_name) => {
                renderered_stage_name_chooser.set_active_id(Some(input_name));
            }

            SampledInput::Nearest(input_name) => {
                renderered_stage_name_chooser.set_active_id(Some(input_name));
            }
            SampledInput::Mipmaps(input_name) => {
                renderered_stage_name_chooser.set_active_id(Some(input_name));
            }
        }

        {
            let renderered_stage_name_chooser = renderered_stage_name_chooser.clone();
            connect!(
                relm,
                renderered_stage_name_chooser,
                connect_changed(_),
                Some(Msg::UpdateRenderedTextureName)
            );
        }

        final_stage_row.add(&final_stage_label);
        final_stage_row.add(&renderered_stage_name_chooser);

        control_container.add(&final_stage_row);

        let start_button = Button::new();
        start_button.set_label("Start");
        start_button.set_hexpand(true);
        connect!(relm, start_button, connect_clicked(_), Some(Msg::Start));

        control_container.add(&start_button);

        window_container.add(&tabs_container);
        window_container.add(&control_container);

        MainPanel {
            model,

            window_container,

            input_list_container,

            input_config_widget_list,

            render_stage_config_list_container,
            render_stage_config_widget_list,
            render_stage_order,

            renderered_stage_name_chooser,

            relm: relm.clone(),
        }
    }
}