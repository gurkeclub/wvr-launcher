use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

use uuid::Uuid;

use gtk::Orientation::Horizontal;
use gtk::{prelude::NotebookExtManual, NotebookExt};
use gtk::{
    Button, ButtonExt, ContainerExt, Label, LabelExt, Notebook, PackType, ReliefStyle, TreeIter,
    TreeModel, TreeModelExt, WidgetExt,
};

use relm::{connect, Cast, Component, ContainerWidget, Relm};

use wvr_data::config::project_config::{BufferPrecision, FilterMode, RenderStageConfig};

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
        (Component<RenderStageConfigView>, gtk::Box),
    >,
) -> (Notebook, Vec<Uuid>) {
    let mut render_stage_order = Vec::new();

    let render_stage_list_container = Notebook::new();
    render_stage_list_container.set_hexpand(true);
    render_stage_list_container.set_vexpand(true);
    render_stage_list_container.set_tab_pos(gtk::PositionType::Left);
    render_stage_list_container.set_show_border(false);
    render_stage_list_container.set_scrollable(true);

    let add_render_stage_button = Button::new();
    add_render_stage_button.set_label("+");
    add_render_stage_button.set_property_margin(4);
    connect!(relm, add_render_stage_button, connect_clicked(_), {
        let render_stage_name = "New layer".to_string();
        let filter_name = "copy_input";
        let render_stage_config = RenderStageConfig {
            name: render_stage_name,
            filter: filter_name.to_string(),
            filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 0.0, 0.0),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            precision: BufferPrecision::U8,
        };
        Some(ConfigPanelMsg::AddRenderStage(render_stage_config))
    });

    add_render_stage_button.show_all();

    render_stage_list_container.set_action_widget(&add_render_stage_button, PackType::End);

    for render_stage_config in render_stage_config_list {
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
        render_stage_list_container.set_tab_reorderable(&wrapper, true);
        {
            let wrapper = wrapper.clone();
            connect!(
                relm,
                render_stage_list_container,
                connect_page_reordered(_, widget, target_index),
                {
                    if widget == &wrapper.clone().upcast::<gtk::Widget>() {
                        println!("moved {:} to {:}", &id, &target_index);

                        Some(ConfigPanelMsg::MoveStage(id, target_index as usize))
                    } else {
                        None
                    }
                }
            );
        }
        page_label_container.show_all();

        render_stage_config_widget_list.insert(id, (render_stage_config_view, wrapper));
        render_stage_order.push(id);
    }

    (render_stage_list_container, render_stage_order)
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
