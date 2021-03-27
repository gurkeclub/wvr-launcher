use std::collections::HashMap;

use uuid::Uuid;

use gdk::RGBA;
use gtk::prelude::NotebookExtManual;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{
    Adjustment, Button, ButtonExt, ContainerExt, EditableSignals, Entry, EntryExt, Label, LabelExt,
    Notebook, ScrolledWindow, StateFlags, WidgetExt,
};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;
use wvr_data::config::project_config::{RenderStageConfig, SampledInput};

pub fn build_list_view(
    relm: &Relm<crate::Win>,
    render_stage_config_widget_list: &mut HashMap<
        Uuid,
        (
            String,
            RenderStageConfig,
            Component<RenderStageConfigView>,
            gtk::Box,
        ),
    >,
    render_stage_config_list: &Vec<RenderStageConfig>,
) -> (gtk::Box, gtk::Box) {
    let render_stage_list_panel = gtk::Box::new(Vertical, 4);
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
        let (id, wrapper, render_stage_config_view) =
            build_render_stage_config_row(relm, &render_stage_config);
        render_stage_list_container.add(&wrapper);
        render_stage_config_widget_list.insert(
            id,
            (
                render_stage_config.name.clone(),
                render_stage_config.clone(),
                render_stage_config_view,
                wrapper,
            ),
        );
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
    render_stage_config: &RenderStageConfig,
) -> (Uuid, gtk::Box, Component<RenderStageConfigView>) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 4);
    let (label_name, label_color) = (
        "Render Stage",
        RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.125,
        },
    );

    let row_label = Label::new(Some(label_name));
    row_label.set_size_request(64, 0);
    row_label.override_background_color(StateFlags::NORMAL, Some(&label_color));

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    connect!(
        relm,
        remove_button,
        connect_clicked(_),
        Some(crate::Msg::RemoveRenderStage(id))
    );

    wrapper.add(&row_label);
    let render_stage_config_view = wrapper.add_widget::<RenderStageConfigView>((
        id.clone(),
        render_stage_config.name.to_string(),
        render_stage_config.clone(),
        relm.clone(),
    ));
    wrapper.add(&remove_button);

    (id, wrapper, render_stage_config_view)
}

#[derive(Msg)]
pub enum RenderStageConfigViewMsg {
    SetName(String),

    UpdateInputList,
}

pub struct RenderStageConfigViewModel {
    parent_relm: Relm<crate::Win>,
    id: Uuid,
    name: String,
    config: RenderStageConfig,
}
pub struct RenderStageConfigView {
    model: RenderStageConfigViewModel,
    relm: Relm<Self>,
    root: gtk::Box,

    input_list_container: gtk::Box,
    input_widget_list: HashMap<String, (Entry, Entry)>,
}

impl RenderStageConfigView {
    pub fn update_input_list(&mut self) {
        self.model.config.inputs.clear();
        for (input_name, (input_type_entry, input_name_entry)) in self.input_widget_list.iter() {
            self.model.config.inputs.insert(
                input_name.clone(),
                match input_type_entry.get_text().as_str() {
                    "Linear" => SampledInput::Linear(input_name_entry.get_text().to_string()),
                    "Nearest" => SampledInput::Nearest(input_name_entry.get_text().to_string()),
                    "Mipmaps" => SampledInput::Mipmaps(input_name_entry.get_text().to_string()),
                    _ => unreachable!(),
                },
            );
        }
    }
}

impl Update for RenderStageConfigView {
    type Model = RenderStageConfigViewModel;
    type ModelParam = (Uuid, String, RenderStageConfig, Relm<crate::Win>);
    type Msg = RenderStageConfigViewMsg;

    fn model(
        _: &Relm<Self>,
        model: (Uuid, String, RenderStageConfig, Relm<crate::Win>),
    ) -> Self::Model {
        RenderStageConfigViewModel {
            id: model.0,
            name: model.1,
            config: model.2,
            parent_relm: model.3,
        }
    }

    fn update(&mut self, event: RenderStageConfigViewMsg) {
        match event {
            RenderStageConfigViewMsg::SetName(new_name) => self.model.name = new_name,
            RenderStageConfigViewMsg::UpdateInputList => self.update_input_list(),
        }

        self.model
            .parent_relm
            .stream()
            .emit(crate::Msg::UpdateRenderStageConfig(
                self.model.id,
                self.model.name.clone(),
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

        let name_row = gtk::Box::new(Horizontal, 8);
        name_row.set_property_margin(8);

        let name_label = Label::new(Some("Name: "));
        name_label.set_xalign(0.0);
        name_label.set_size_request(48, 0);

        let name_entry = Entry::new();
        name_entry.set_text(&model.name);
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

        let tabs_container = Notebook::new();
        tabs_container.set_property_margin(8);

        // Begin of input list composition panel
        let input_row = gtk::Box::new(Vertical, 8);
        input_row.set_property_margin(8);

        let input_list_container = gtk::Box::new(Vertical, 8);

        for (input_name, input_value) in &model.config.inputs {
            let (input_wrapper, input_type_entry, input_value_entry) =
                build_input_row(relm, &input_name, &input_value);
            input_widget_list.insert(input_name.clone(), (input_type_entry, input_value_entry));
            input_list_container.add(&input_wrapper);
        }

        input_row.add(&input_list_container);

        tabs_container.append_page(&input_row, Some(&Label::new(Some("Inputs"))));

        root.add(&name_row);
        root.add(&tabs_container);

        Self {
            relm: relm.clone(),
            model,
            root,
            input_list_container,
            input_widget_list,
        }
    }
}

fn build_input_row(
    relm: &Relm<RenderStageConfigView>,
    uniform_name: &str,
    input_value: &SampledInput,
) -> (gtk::Box, Entry, Entry) {
    let wrapper = gtk::Box::new(Horizontal, 4);

    let uniform_name_label = Label::new(Some(uniform_name));
    uniform_name_label.set_xalign(0.0);
    uniform_name_label.set_size_request(48, 0);

    let input_type_entry = Entry::new();
    input_type_entry.set_hexpand(true);

    let input_name_entry = Entry::new();
    input_name_entry.set_hexpand(true);

    match input_value {
        SampledInput::Linear(input_name) => {
            input_type_entry.set_text("Linear");
            input_name_entry.set_text(input_name);
        }

        SampledInput::Nearest(input_name) => {
            input_type_entry.set_text("Nearest");
            input_name_entry.set_text(input_name);
        }
        SampledInput::Mipmaps(input_name) => {
            input_type_entry.set_text("Mipmaps");
            input_name_entry.set_text(input_name);
        }
    }

    {
        let input_name_entry = input_name_entry.clone();
        connect!(
            relm,
            input_name_entry,
            connect_changed(_),
            Some(RenderStageConfigViewMsg::UpdateInputList)
        );
    }
    {
        let input_type_entry = input_type_entry.clone();
        connect!(
            relm,
            input_type_entry,
            connect_changed(_),
            Some(RenderStageConfigViewMsg::UpdateInputList)
        );
    }
    wrapper.add(&uniform_name_label);
    wrapper.add(&input_type_entry);
    wrapper.add(&input_name_entry);

    return (wrapper, input_type_entry, input_name_entry);
}
