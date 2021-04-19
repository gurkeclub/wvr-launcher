use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Result;

use uuid::Uuid;

use gtk::Orientation::Horizontal;
use gtk::{prelude::NotebookExtManual, NotebookExt};
use gtk::{
    Button, ButtonExt, ContainerExt, Label, LabelExt, Notebook, PackType, ReliefStyle, TreeIter,
    TreeModel, TreeModelExt, WidgetExt,
};

use relm::{connect, Component, ContainerWidget, Relm};

use wvr_data::config::project_config::{FilterConfig, RenderStageConfig};

pub mod input;
pub mod variable;
pub mod view;

use crate::config_panel::msg::ConfigPanelMsg;
use crate::config_panel::view::ConfigPanel;
use view::RenderStageConfigView;

pub fn build_list_view(
    relm: &Relm<ConfigPanel>,
    project_path: &Path,
    render_stage_config_list: &[RenderStageConfig],
    input_choice_list: &[String],
    render_stage_config_widget_list: &mut HashMap<
        Uuid,
        (usize, Component<RenderStageConfigView>, gtk::Box),
    >,
) -> Notebook {
    let render_stage_list_container = Notebook::new();
    render_stage_list_container.set_hexpand(true);
    render_stage_list_container.set_vexpand(true);
    render_stage_list_container.set_tab_pos(gtk::PositionType::Left);
    render_stage_list_container.set_show_border(false);
    render_stage_list_container.set_scrollable(true);

    let add_render_stage_button = Button::new();
    add_render_stage_button.set_label("+");
    add_render_stage_button.set_property_margin(4);
    connect!(
        relm,
        add_render_stage_button,
        connect_clicked(_),
        Some(ConfigPanelMsg::AddRenderStage)
    );

    add_render_stage_button.show_all();

    render_stage_list_container.set_action_widget(&add_render_stage_button, PackType::End);

    for (render_stage_index, render_stage_config) in render_stage_config_list.iter().enumerate() {
        let (id, wrapper, render_stage_config_view) = build_render_stage_config_row(
            relm,
            project_path,
            &render_stage_config,
            &input_choice_list,
        );
        let page_label_container = gtk::Box::new(Horizontal, 4);
        page_label_container.set_property_margin(0);

        let page_label = Label::new(Some(&render_stage_config.name));
        page_label.set_xalign(0.0);
        page_label.set_hexpand(true);

        let remove_button = Button::new();
        remove_button.set_relief(ReliefStyle::None);
        remove_button.set_label("x");
        {
            connect!(
                relm,
                remove_button,
                connect_clicked(_),
                Some(ConfigPanelMsg::RemoveRenderStage(id))
            );
        }

        page_label_container.add(&remove_button);
        page_label_container.add(&page_label);

        render_stage_list_container.append_page(&wrapper, Some(&page_label_container));
        page_label_container.show_all();

        render_stage_config_widget_list
            .insert(id, (render_stage_index, render_stage_config_view, wrapper));
    }

    render_stage_list_container
}

pub fn build_render_stage_config_row(
    relm: &Relm<ConfigPanel>,
    project_path: &Path,
    render_stage_config: &RenderStageConfig,
    input_choice_list: &[String],
) -> (Uuid, gtk::Box, Component<RenderStageConfigView>) {
    let id = Uuid::new_v4();
    let wrapper = gtk::Box::new(Horizontal, 2);

    let render_stage_config_view = wrapper.add_widget::<RenderStageConfigView>((
        id,
        project_path.to_owned(),
        render_stage_config.clone(),
        input_choice_list.to_vec(),
        relm.clone(),
    ));

    (id, wrapper, render_stage_config_view)
}

pub fn load_available_filter_list(
    project_path: &Path,
) -> Result<HashMap<String, (PathBuf, FilterConfig)>> {
    let mut available_filter_list = HashMap::new();

    let project_filter_folder_path = project_path.join("filters");
    let wvr_filter_folder_path = wvr_data::get_filters_path();

    // Load filters from project

    if project_filter_folder_path.exists() {
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
    }

    // Load filters provided by wvr
    if wvr_filter_folder_path.exists() {
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
