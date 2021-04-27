use gtk::prelude::{GtkListStoreExtManual, TreeSortableExtManual};
use gtk::Orientation::Horizontal;
use gtk::{ComboBoxExt, ComboBoxText, ContainerExt, SortColumn, SortType, WidgetExt};

use relm::{connect, Relm};

use wvr_data::config::project_config::SampledInput;

use super::view::{RenderStageConfigView, RenderStageConfigViewMsg};

pub fn build_input_row(
    relm: &Relm<RenderStageConfigView>,
    input_choice_list: &[String],
    input_name: &str,
    input_value: &SampledInput,
) -> (gtk::Box, ComboBoxText, ComboBoxText) {
    let outer_wrapper = gtk::Box::new(Horizontal, 8);

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
    input_name_chooser.set_hexpand(true);
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
        let input_name = input_name.to_owned();

        let input_name_chooser = input_name_chooser.clone();

        let input_type_chooser = input_type_chooser.clone();
        input_type_chooser.set_tooltip_text(Some("Sampling method"));

        connect!(
            relm,
            input_type_chooser,
            connect_changed(input_type_chooser),
            {
                let input_value = match input_type_chooser.get_active_id().unwrap().as_str() {
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
                Some(RenderStageConfigViewMsg::UpdateInput(
                    input_name.clone(),
                    input_value,
                ))
            }
        );
    }

    {
        let input_name = input_name.to_owned();
        let input_name_chooser = input_name_chooser.clone();
        let input_type_chooser = input_type_chooser.clone();

        connect!(
            relm,
            input_name_chooser,
            connect_changed(input_name_chooser),
            {
                let input_value = match input_type_chooser.get_active_id().unwrap().as_str() {
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
                Some(RenderStageConfigViewMsg::UpdateInput(
                    input_name.clone(),
                    input_value,
                ))
            }
        );
    }

    outer_wrapper.add(&input_name_chooser);
    outer_wrapper.add(&input_type_chooser);

    (outer_wrapper, input_type_chooser, input_name_chooser)
}
