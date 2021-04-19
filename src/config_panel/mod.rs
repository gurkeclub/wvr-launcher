use wvr_data::config::project_config::ProjectConfig;

pub mod msg;
pub mod view;

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
