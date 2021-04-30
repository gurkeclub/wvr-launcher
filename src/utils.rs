use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use nfd2::Response;

use wvr_data::config::project_config::{
    BufferPrecision, FilterMode, ProjectConfig, RenderStageConfig, SampledInput, ViewConfig,
};
use wvr_data::config::server_config::ServerConfig;

pub fn create_project(project_config_path: &Path) -> Option<ProjectConfig> {
    std::fs::create_dir_all(&project_config_path).unwrap();
    std::fs::create_dir_all(&project_config_path.join("filters")).unwrap();

    let patterns_stage = RenderStageConfig {
        name: "Patterns".to_owned(),
        filter: "generate/dots".to_owned(),
        filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 1.0, 1.0),
        inputs: HashMap::new(),
        variables: HashMap::new(),
        precision: BufferPrecision::F32,
    };

    let mut final_stage_input_list = HashMap::new();
    final_stage_input_list.insert(
        "iChannel0".to_owned(),
        SampledInput::Linear("Patterns".to_owned()),
    );

    let final_stage = RenderStageConfig {
        name: "FinalStage".to_owned(),
        filter: "generic/copy".to_owned(),
        filter_mode_params: FilterMode::Rectangle(0.0, 0.0, 1.0, 1.0),
        inputs: final_stage_input_list,
        variables: HashMap::new(),
        precision: BufferPrecision::F32,
    };

    let project_config = ProjectConfig {
        bpm: 89.0,
        view: ViewConfig {
            width: 640,
            height: 480,
            fullscreen: false,
            dynamic: true,
            vsync: true,
            screenshot: false,
            screenshot_path: PathBuf::from("output/"),
            target_fps: 60.0,
            locked_speed: false,
        },
        server: ServerConfig {
            ip: "localhost".to_owned(),
            port: 3000,
            enable: false,
        },
        inputs: HashMap::new(),
        render_chain: vec![patterns_stage],
        final_stage,
    };
    if let Ok(mut project_config_file) =
        std::fs::File::create(&project_config_path.join("config.json"))
    {
        project_config_file
            .write_all(
                &serde_json::ser::to_string_pretty(&project_config)
                    .unwrap()
                    .into_bytes(),
            )
            .unwrap();

        Some(project_config)
    } else {
        None
    }
}

pub fn get_config() -> std::option::Option<(PathBuf, ProjectConfig)> {
    let wvr_data_path = wvr_data::get_data_path();

    let mut config_path = None;
    let projects_path = wvr_data_path.join("projects");

    while config_path.is_none() {
        match nfd2::open_file_dialog(None, Some(&projects_path)).expect("oh no") {
            Response::Okay(file_path) => config_path = Some(file_path),
            Response::OkayMultiple(_) => (),
            Response::Cancel => return None,
        }
    }

    let config_path = config_path.unwrap();

    let project_path = config_path.parent().unwrap().to_owned();
    let config: ProjectConfig = if let Ok(file) = File::open(&config_path) {
        serde_json::from_reader::<File, ProjectConfig>(file).unwrap()
    } else {
        panic!("Could not find config file {:?}", project_path);
    };

    Some((project_path, config))
}
