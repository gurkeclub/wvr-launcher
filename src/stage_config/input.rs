use gtk::prelude::{GtkListStoreExtManual, TreeSortableExtManual};
use gtk::Orientation::Horizontal;
use gtk::{
    ComboBoxExt, ComboBoxText, ContainerExt, Label, LabelExt, SortColumn, SortType, StateFlags,
    WidgetExt,
};

use gdk::RGBA;

use relm::{connect, Relm};

use wvr_data::config::project_config::SampledInput;

use super::{RenderStageConfigView, RenderStageConfigViewMsg};

pub fn build_input_row(
    relm: &Relm<RenderStageConfigView>,
    input_choice_list: &[String],
    uniform_name: &str,
    input_value: &SampledInput,
) -> (gtk::Box, ComboBoxText, ComboBoxText) {
    let outer_wrapper = gtk::Box::new(Horizontal, 2);
    outer_wrapper.override_background_color(
        StateFlags::NORMAL,
        Some(&RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0625,
        }),
    );

    let wrapper = gtk::Box::new(Horizontal, 2);
    wrapper.set_property_margin(4);

    let uniform_name_label = Label::new(Some(uniform_name));
    uniform_name_label.set_xalign(0.0);
    uniform_name_label.set_size_request(48, 0);

    let padding = gtk::Box::new(Horizontal, 0);
    padding.set_hexpand(true);

    let input_type_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
    for name in ["Linear", "Nearest", "Mipmaps"].iter() {
        input_type_store.insert_with_values(None, &[0, 1], &[name, name]);
    }
    input_type_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
    input_type_store.set_default_sort_func(&super::list_store_sort_function);

    let input_type_chooser = gtk::ComboBoxText::new();
    input_type_chooser.set_model(Some(&input_type_store));

    input_type_chooser.set_id_column(0);
    input_type_chooser.set_entry_text_column(1);

    let input_name_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::String]);
    for name in input_choice_list {
        input_name_store.insert_with_values(None, &[0, 1], &[name, name]);
    }
    input_name_store.set_sort_column_id(SortColumn::Index(0), SortType::Ascending);
    input_name_store.set_default_sort_func(&super::list_store_sort_function);

    let input_name_chooser = gtk::ComboBoxText::new();
    input_name_chooser.set_model(Some(&input_name_store));

    input_name_chooser.set_id_column(0);
    input_name_chooser.set_entry_text_column(1);

    match input_value {
        SampledInput::Linear(input_name) => {
            input_type_chooser.set_active_id(Some("Linear"));
            input_name_chooser.set_active_id(Some(input_name));
        }

        SampledInput::Nearest(input_name) => {
            input_type_chooser.set_active_id(Some("Nearest"));
            input_name_chooser.set_active_id(Some(input_name));
        }
        SampledInput::Mipmaps(input_name) => {
            input_type_chooser.set_active_id(Some("Mipmaps"));
            input_name_chooser.set_active_id(Some(input_name));
        }
    }
    {
        let input_type_chooser = input_type_chooser.clone();
        connect!(
            relm,
            input_type_chooser,
            connect_changed(_),
            Some(RenderStageConfigViewMsg::UpdateInputList)
        );
    }

    {
        let input_name_chooser = input_name_chooser.clone();
        connect!(
            relm,
            input_name_chooser,
            connect_changed(_),
            Some(RenderStageConfigViewMsg::UpdateInputList)
        );
    }

    wrapper.add(&uniform_name_label);
    wrapper.add(&padding);
    wrapper.add(&input_type_chooser);
    wrapper.add(&input_name_chooser);

    outer_wrapper.add(&wrapper);

    (outer_wrapper, input_type_chooser, input_name_chooser)
}
