use bevy::prelude::*;
use rand::Rng;

use crate::cf_tool;
use crate::constants::*;
use crate::resources::*;
use crate::systems;

/// カメラ制御プラグイン
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseDragState>()
            .add_systems(
                Update,
                (
                    systems::camera_zoom,
                    systems::camera_drag_rotation,
                    systems::camera_keyboard_rotation,
                    systems::camera_keyboard_pan,
                ),
            );
    }
}

/// UI制御プラグイン
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsMenuState>()
            .add_systems(
                Update,
                (
                    systems::toggle_settings_menu,
                    systems::handle_setting_buttons,
                    systems::update_setting_value_texts,
                    systems::update_item_slot_display,
                    systems::update_item_slot_highlight,
                    systems::handle_item_slot_click,
                ),
            );
    }
}

/// ゲームロジックプラグイン
pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoxMoveMode>()
            .init_resource::<SelectedItemSlot>()
            .add_systems(
                Update,
                (
                    systems::block_hover_highlight,
                    systems::block_click_handler,
                    systems::handle_fox_action_buttons,
                    systems::fox_follow_cursor,
                    cf_tool::timer::update_timers,
                    cf_tool::timer::update_timer_ui,
                ),
            );
    }
}

/// 天候システムプラグイン
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        let mut rng = rand::rng();
        app.insert_resource(WeatherState {
            is_raining: false,
            time_until_change: rng.random_range(
                WEATHER_INITIAL_CHANGE_MIN..WEATHER_INITIAL_CHANGE_MAX,
            ),
        })
        .add_systems(
            Update,
            (systems::update_weather, systems::spawn_rain, systems::update_rain),
        );
    }
}
