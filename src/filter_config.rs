use std::collections::HashMap;

use uuid::Uuid;

use gdk::RGBA;
use gtk::prelude::NotebookExtManual;
use gtk::Orientation::{Horizontal, Vertical};
use gtk::{Adjustment, Button, ButtonExt, ContainerExt, EditableSignals, Entry, EntryExt, Label, LabelExt, Notebook, ScrolledWindow, StateFlags, WidgetExt};

use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;
use wvr_data::config::project_config::FilterConfig;

pub fn build_list_view(
    relm: &Relm<crate::Win>,
    filter_config_widget_list: &mut HashMap<Uuid, (String, FilterConfig, Component<FilterConfigView>, gtk::Box)>,
    filter_config_list: &HashMap<String, FilterConfig>,
) -> (gtk::Box, gtk::Box) {
    let filter_list_panel = gtk::Box::new(Vertical, 4);
    filter_list_panel.set_property_margin(4);

    let filter_list_control_container = gtk::Box::new(Horizontal, 8);
    filter_list_control_container.set_property_margin(8);

    let add_filter_button = Button::new();
    add_filter_button.set_label("Add Filter");
    add_filter_button.set_hexpand(true);
    connect!(relm, add_filter_button, connect_clicked(_), Some(crate::Msg::AddFilter));

    filter_list_control_container.add(&add_filter_button);

    let filter_list_container = gtk::Box::new(Vertical, 16);
    filter_list_container.set_property_margin(8);

    for (filter_name, filter_config) in filter_config_list.iter() {
        let (id, wrapper, filter_config_view) = build_filter_config_row(relm, filter_name, &filter_config);
        filter_list_container.add(&wrapper);
        filter_config_widget_list.insert(id, (filter_name.clone(), filter_config.clone(), filter_config_view, wrapper));
    }

    let filter_list_container_wrapper = ScrolledWindow::new(
        Some(&Adjustment::new(320.0, 320.0, 10000.0, 1.0, 1.0, 1.0)),
        Some(&Adjustment::new(320.0, 320.0, 100000.0, 0.0, 0.0, 1.0)),
    );
    filter_list_container_wrapper.set_size_request(480, 320);
    filter_list_container_wrapper.set_hexpand(true);
    filter_list_container_wrapper.set_vexpand(true);
    filter_list_container_wrapper.add(&filter_list_container);

    filter_list_panel.add(&filter_list_container_wrapper);
    filter_list_panel.add(&filter_list_control_container);

    (filter_list_panel, filter_list_container)
}

pub fn build_filter_config_row(relm: &Relm<crate::Win>, filter_name: &str, filter_config: &FilterConfig) -> (Uuid, gtk::Box, Component<FilterConfigView>) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 4);
    let (label_name, label_color) = (
        "Filter",
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
    connect!(relm, remove_button, connect_clicked(_), Some(crate::Msg::RemoveFilter(id)));

    wrapper.add(&row_label);
    let filter_config_view = wrapper.add_widget::<FilterConfigView>((id.clone(), filter_name.to_string(), filter_config.clone(), relm.clone()));
    wrapper.add(&remove_button);

    (id, wrapper, filter_config_view)
}

#[derive(Msg)]
pub enum FilterConfigViewMsg {
    SetName(String),

    UpdateVertexPathList,
    AddVertexPath,
    RemoveVertexPath(gtk::Box, Entry),

    UpdateFragmentPathList,
    AddFragmentPath,
    RemoveFragmentPath(gtk::Box, Entry),

    UpdateInputNameList,
    RemoveInput(gtk::Box, Entry),
    AddInput,
}

pub struct FilterConfigViewModel {
    parent_relm: Relm<crate::Win>,
    id: Uuid,
    name: String,
    config: FilterConfig,
}
pub struct FilterConfigView {
    model: FilterConfigViewModel,
    relm: Relm<Self>,
    root: gtk::Box,

    input_name_list_container: gtk::Box,
    input_name_widget_list: Vec<Entry>,

    vertex_shader_list_container: gtk::Box,
    vertex_shader_path_widget_list: Vec<Entry>,

    fragment_shader_list_container: gtk::Box,
    fragment_shader_path_widget_list: Vec<Entry>,
}

impl FilterConfigView {
    pub fn update_input_name_list(&mut self) {
        self.model.config.inputs = self.input_name_widget_list.iter().map(|entry| entry.get_text().to_string()).collect();
    }

    pub fn update_vertex_shader_list(&mut self) {
        self.model.config.vertex_shader = self.vertex_shader_path_widget_list.iter().map(|entry| entry.get_text().to_string()).collect();
    }

    pub fn update_fragment_shader_list(&mut self) {
        self.model.config.fragment_shader = self.fragment_shader_path_widget_list.iter().map(|entry| entry.get_text().to_string()).collect();
    }
}

impl Update for FilterConfigView {
    type Model = FilterConfigViewModel;
    type ModelParam = (Uuid, String, FilterConfig, Relm<crate::Win>);
    type Msg = FilterConfigViewMsg;

    fn model(_: &Relm<Self>, model: (Uuid, String, FilterConfig, Relm<crate::Win>)) -> Self::Model {
        FilterConfigViewModel {
            id: model.0,
            name: model.1,
            config: model.2,
            parent_relm: model.3,
        }
    }

    fn update(&mut self, event: FilterConfigViewMsg) {
        match event {
            FilterConfigViewMsg::SetName(new_name) => self.model.name = new_name,

            FilterConfigViewMsg::AddVertexPath => {
                let (vertex_path_wrapper, vertex_path_entry) = build_shader_file_row(&self.relm, "", false);
                self.vertex_shader_path_widget_list.push(vertex_path_entry);
                self.vertex_shader_list_container.add(&vertex_path_wrapper);
                self.vertex_shader_list_container.show_all();
            }
            FilterConfigViewMsg::UpdateVertexPathList => self.update_vertex_shader_list(),
            FilterConfigViewMsg::RemoveVertexPath(vertex_path_wrapper, vertex_path_entry) => {
                self.vertex_shader_list_container.remove(&vertex_path_wrapper);
                self.vertex_shader_path_widget_list.remove(
                    self.vertex_shader_path_widget_list
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| *item.clone() == vertex_path_entry.clone())
                        .map(|(idx, _)| idx)
                        .next()
                        .unwrap(),
                );

                self.update_vertex_shader_list();

                self.vertex_shader_list_container.show_all();
            }

            FilterConfigViewMsg::AddFragmentPath => {
                let (fragment_path_wrapper, fragment_path_entry) = build_shader_file_row(&self.relm, "", true);
                self.fragment_shader_path_widget_list.push(fragment_path_entry);
                self.fragment_shader_list_container.add(&fragment_path_wrapper);
                self.fragment_shader_list_container.show_all();
            }
            FilterConfigViewMsg::UpdateFragmentPathList => self.update_fragment_shader_list(),
            FilterConfigViewMsg::RemoveFragmentPath(fragment_path_wrapper, fragment_path_entry) => {
                self.fragment_shader_list_container.remove(&fragment_path_wrapper);
                self.fragment_shader_path_widget_list.remove(
                    self.fragment_shader_path_widget_list
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| fragment_path_entry.eq(*item))
                        .map(|(idx, _)| {
                            println!("{:}", idx);
                            idx
                        })
                        .next()
                        .unwrap(),
                );

                self.update_fragment_shader_list();

                self.fragment_shader_list_container.show_all();
            }

            FilterConfigViewMsg::AddInput => {
                let (input_name_wrapper, input_name_entry) = build_input_name_row(&self.relm, "");
                self.input_name_widget_list.push(input_name_entry);
                self.input_name_list_container.add(&input_name_wrapper);
                self.input_name_list_container.show_all();
            }
            FilterConfigViewMsg::UpdateInputNameList => self.update_input_name_list(),
            FilterConfigViewMsg::RemoveInput(input_name_wrapper, input_name_entry) => {
                self.input_name_list_container.remove(&input_name_wrapper);
                self.input_name_widget_list.remove(
                    self.input_name_widget_list
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| *item.clone() == input_name_entry.clone())
                        .map(|(idx, _)| idx)
                        .next()
                        .unwrap(),
                );

                self.update_input_name_list();

                self.input_name_list_container.show_all();
            }
        }

        self.model
            .parent_relm
            .stream()
            .emit(crate::Msg::UpdateFilterConfig(self.model.id, self.model.name.clone(), self.model.config.clone()));
    }
}

impl Widget for FilterConfigView {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let mut input_name_widget_list = Vec::new();
        let mut vertex_shader_path_widget_list = Vec::new();
        let mut fragment_shader_path_widget_list = Vec::new();

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
        connect!(relm, name_entry, connect_changed(val), Some(FilterConfigViewMsg::SetName(val.get_text().to_string())));

        name_row.add(&name_label);
        name_row.add(&name_entry);

        let tabs_container = Notebook::new();
        tabs_container.set_property_margin(8);

        // Begin of input list composition panel
        let input_name_row = gtk::Box::new(Vertical, 8);
        input_name_row.set_property_margin(8);

        let input_name_list_container = gtk::Box::new(Vertical, 8);

        for input_name in &model.config.inputs {
            let (input_name_wrapper, input_name_entry) = build_input_name_row(relm, &input_name);
            input_name_widget_list.push(input_name_entry);
            input_name_list_container.add(&input_name_wrapper);
        }

        let add_input_button = Button::new();
        add_input_button.set_label("Add input code path");
        add_input_button.set_hexpand(true);
        connect!(relm, add_input_button, connect_clicked(_), Some(FilterConfigViewMsg::AddInput));

        input_name_row.add(&input_name_list_container);
        input_name_row.add(&add_input_button);

        // Begin of vertex shader composition panel
        let vertex_shader_row = gtk::Box::new(Vertical, 8);
        vertex_shader_row.set_property_margin(8);

        let vertex_shader_list_container = gtk::Box::new(Vertical, 8);

        for vertex_shader_path in &model.config.vertex_shader {
            let (vertex_path_wrapper, vertex_path_entry) = build_shader_file_row(relm, &vertex_shader_path, false);
            vertex_shader_path_widget_list.push(vertex_path_entry);
            vertex_shader_list_container.add(&vertex_path_wrapper);
        }

        let add_vertex_button = Button::new();
        add_vertex_button.set_label("Add vertex code path");
        add_vertex_button.set_hexpand(true);
        connect!(relm, add_vertex_button, connect_clicked(_), Some(FilterConfigViewMsg::AddVertexPath));

        vertex_shader_row.add(&vertex_shader_list_container);
        vertex_shader_row.add(&add_vertex_button);

        // Begin of fragment shader composition panel
        let fragment_shader_row = gtk::Box::new(Vertical, 8);
        fragment_shader_row.set_property_margin(8);

        let fragment_shader_list_container = gtk::Box::new(Vertical, 8);

        for fragment_shader_path in &model.config.fragment_shader {
            let (fragment_path_wrapper, fragment_path_entry) = build_shader_file_row(relm, &fragment_shader_path, true);
            fragment_shader_path_widget_list.push(fragment_path_entry);
            fragment_shader_list_container.add(&fragment_path_wrapper);
        }

        let add_fragment_button = Button::new();
        add_fragment_button.set_label("Add fragment code path");
        add_fragment_button.set_hexpand(true);
        connect!(relm, add_fragment_button, connect_clicked(_), Some(FilterConfigViewMsg::AddFragmentPath));

        fragment_shader_row.add(&fragment_shader_list_container);
        fragment_shader_row.add(&add_fragment_button);

        tabs_container.append_page(&input_name_row, Some(&Label::new(Some("Inputs"))));
        tabs_container.append_page(&vertex_shader_row, Some(&Label::new(Some("Vertex shader"))));
        tabs_container.append_page(&fragment_shader_row, Some(&Label::new(Some("Fragment shader"))));

        root.add(&name_row);
        root.add(&tabs_container);

        Self {
            relm: relm.clone(),
            model,
            root,
            input_name_list_container,
            input_name_widget_list,
            vertex_shader_path_widget_list,
            vertex_shader_list_container,
            fragment_shader_path_widget_list,
            fragment_shader_list_container,
        }
    }
}

fn build_input_name_row(relm: &Relm<FilterConfigView>, path: &str) -> (gtk::Box, Entry) {
    let wrapper = gtk::Box::new(Horizontal, 4);

    let input_name_entry = Entry::new();
    input_name_entry.set_hexpand(true);
    input_name_entry.set_text(path);

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    let input_name_entry = input_name_entry.clone();
    let wrapper = wrapper.clone();

    {
        let wrapper = wrapper.clone();
        let input_name_entry = input_name_entry.clone();
        connect!(relm, input_name_entry, connect_changed(_), Some(FilterConfigViewMsg::UpdateInputNameList));

        connect!(
            relm,
            remove_button,
            connect_clicked(_),
            Some(FilterConfigViewMsg::RemoveInput(wrapper.clone(), input_name_entry.clone()))
        );
    }
    wrapper.add(&input_name_entry);
    wrapper.add(&remove_button);

    return (wrapper, input_name_entry);
}

fn build_shader_file_row(relm: &Relm<FilterConfigView>, path: &str, fragment_shader: bool) -> (gtk::Box, Entry) {
    let wrapper = gtk::Box::new(Horizontal, 4);

    let shader_path_entry = Entry::new();
    shader_path_entry.set_hexpand(true);
    shader_path_entry.set_text(path);

    let remove_button = Button::new();
    remove_button.set_label("Delete");
    if fragment_shader {
        let shader_path_entry = shader_path_entry.clone();
        let wrapper = wrapper.clone();

        connect!(relm, shader_path_entry, connect_changed(_), Some(FilterConfigViewMsg::UpdateFragmentPathList));

        connect!(
            relm,
            remove_button,
            connect_clicked(_),
            Some(FilterConfigViewMsg::RemoveFragmentPath(wrapper.clone(), shader_path_entry.clone()))
        );
    } else {
        let shader_path_entry = shader_path_entry.clone();
        let wrapper = wrapper.clone();

        connect!(relm, shader_path_entry, connect_changed(_), Some(FilterConfigViewMsg::UpdateVertexPathList));

        connect!(
            relm,
            remove_button,
            connect_clicked(_),
            Some(FilterConfigViewMsg::RemoveVertexPath(wrapper.clone(), shader_path_entry.clone()))
        );
    }

    wrapper.add(&shader_path_entry);
    wrapper.add(&remove_button);

    return (wrapper, shader_path_entry);
}
