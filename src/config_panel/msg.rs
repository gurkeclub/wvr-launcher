use std::convert::TryInto;
use std::sync::mpsc::Sender;

use uuid::Uuid;

use relm_derive::Msg;

use wvr_com::data::{Message, RenderStageUpdate, SetInfo};
use wvr_data::config::project_config::{FilterMode, InputConfig, SampledInput};
use wvr_data::DataHolder;

use super::view::ConfigPanel;

#[derive(Msg, Debug)]
pub enum ConfigPanelMsg {
    SetBpm(f64),
    SetWidth(i64),
    SetHeight(i64),
    SetTargetFps(f64),
    SetDynamicResolution(bool),
    SetVSync(bool),
    SetScreenshot(bool),
    SetFullscreen(bool),
    SetLockedSpeed(bool),

    SetServerIp(String),
    SetServerPort(i64),
    SetServerEnabled(bool),

    AddPictureInput,
    AddCamInput,
    AddVideoInput,
    AddMidiInput,
    UpdateInputConfig(Uuid, String, InputConfig),
    RemoveInput(Uuid),

    AddRenderStage,
    UpdateRenderStageFilter(Uuid, String),
    UpdateRenderStageFilterModeParams(Uuid, FilterMode),
    UpdateRenderStageVariable(Uuid, String, DataHolder),
    UpdateRenderStageInput(Uuid, String, SampledInput),
    UpdateRenderStageName(Uuid, String),
    RemoveRenderStage(Uuid),

    SetControlChannel(Sender<Message>),

    UpdateRenderedTextureName,

    Save,
}

impl ConfigPanelMsg {
    pub fn to_wvr_message(&self, config_panel: &ConfigPanel) -> Option<Message> {
        match &self {
            ConfigPanelMsg::SetBpm(bpm) => Some(Message::Set(SetInfo::Bpm(*bpm))),
            ConfigPanelMsg::SetWidth(width) => Some(Message::Set(SetInfo::Width(*width as usize))),
            ConfigPanelMsg::SetHeight(height) => {
                Some(Message::Set(SetInfo::Height(*height as usize)))
            }
            ConfigPanelMsg::SetTargetFps(target_fps) => {
                Some(Message::Set(SetInfo::TargetFps(*target_fps)))
            }
            ConfigPanelMsg::SetDynamicResolution(dynamic_resolution) => Some(Message::Set(
                SetInfo::DynamicResolution(*dynamic_resolution),
            )),
            ConfigPanelMsg::SetVSync(vsync) => Some(Message::Set(SetInfo::VSync(*vsync))),
            ConfigPanelMsg::SetScreenshot(screenshot) => {
                Some(Message::Set(SetInfo::Screenshot(*screenshot)))
            }
            ConfigPanelMsg::SetFullscreen(fullscreen) => {
                Some(Message::Set(SetInfo::Fullscreen(*fullscreen)))
            }
            ConfigPanelMsg::SetLockedSpeed(locked_speed) => {
                Some(Message::Set(SetInfo::LockedSpeed(*locked_speed)))
            }

            ConfigPanelMsg::AddPictureInput => None,
            ConfigPanelMsg::AddCamInput => None,
            ConfigPanelMsg::AddVideoInput => None,
            ConfigPanelMsg::AddMidiInput => None,
            ConfigPanelMsg::UpdateInputConfig(input_id, input_name, input_config) => None,
            ConfigPanelMsg::RemoveInput(input_id) => None,

            ConfigPanelMsg::AddRenderStage => {
                if let Some(render_stage_config) = config_panel.model.config.render_chain.last() {
                    Some(Message::AddRenderStage(render_stage_config.clone()))
                } else {
                    None
                }
            }
            ConfigPanelMsg::UpdateRenderStageFilter(stage_id, filter_name) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::UpdateRenderStage(
                        *stage_index,
                        RenderStageUpdate::Filter(filter_name.clone()),
                    ))
                } else {
                    None
                }
            }
            ConfigPanelMsg::UpdateRenderStageFilterModeParams(stage_id, filter_mode_params) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::UpdateRenderStage(
                        *stage_index,
                        RenderStageUpdate::FilterModeParams(filter_mode_params.clone()),
                    ))
                } else {
                    None
                }
            }
            ConfigPanelMsg::UpdateRenderStageVariable(stage_id, variable_name, variable_value) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::UpdateRenderStage(
                        *stage_index,
                        RenderStageUpdate::Variable(variable_name.clone(), variable_value.clone()),
                    ))
                } else {
                    None
                }
            }
            ConfigPanelMsg::UpdateRenderStageInput(stage_id, input_name, input) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::UpdateRenderStage(
                        *stage_index,
                        RenderStageUpdate::Input(input_name.clone(), input.clone()),
                    ))
                } else {
                    None
                }
            }
            ConfigPanelMsg::UpdateRenderStageName(stage_id, name) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::UpdateRenderStage(
                        *stage_index,
                        RenderStageUpdate::Name(name.clone()),
                    ))
                } else {
                    None
                }
            }
            ConfigPanelMsg::RemoveRenderStage(stage_id) => {
                if let Some((stage_index, _, _)) =
                    config_panel.render_stage_config_widget_list.get(stage_id)
                {
                    Some(Message::RemoveRenderStage(*stage_index))
                } else {
                    None
                }
            }

            ConfigPanelMsg::UpdateRenderedTextureName => {
                if let Some(final_stage_input) = config_panel
                    .model
                    .config
                    .final_stage
                    .inputs
                    .get("iChannel0")
                {
                    Some(Message::UpdateFinalStage(RenderStageUpdate::Input(
                        "iChannel0".to_string(),
                        final_stage_input.clone(),
                    )))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
