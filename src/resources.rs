use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::components::ItemType;

// ========================================
// Game State Resources
// ========================================

/// マウスドラッグの状態を追跡するリソース
#[derive(Resource, Default)]
pub struct MouseDragState {
    pub is_dragging: bool,
    pub last_position: Option<Vec2>,
}

/// Fox移動モードの状態を追跡するリソース
#[derive(Resource, Default)]
pub struct FoxMoveMode {
    pub is_active: bool,
    pub is_holding: bool,
    pub fox_entity: Option<Entity>,
}

/// Possession（憑依）モードの状態を追跡するリソース
#[derive(Resource, Default)]
pub struct PossessionMode {
    pub is_active: bool,
    pub fox_entity: Option<Entity>,
    pub camera_offset: Vec3,
    pub previous_camera_transform: Option<Transform>,
}

/// ダッシュ入力のダブルタップ検出用リソース
#[derive(Resource)]
pub struct DashInputState {
    pub is_dashing: bool,
    pub last_tap_time: Option<f32>,
    pub last_key: Option<KeyCode>,
    pub dash_timeout: f32,
}

impl Default for DashInputState {
    fn default() -> Self {
        Self {
            is_dashing: false,
            last_tap_time: None,
            last_key: None,
            dash_timeout: 0.3, // 300msの間にダブルタップ
        }
    }
}

/// 選択されたアイテムスロットを追跡するリソース
#[derive(Resource, Default)]
pub struct SelectedItemSlot {
    pub slot_index: Option<usize>,
    pub item_type: Option<ItemType>,
}

/// 設定メニューの状態を追跡するリソース
#[derive(Resource, Default)]
pub struct SettingsMenuState {
    pub is_open: bool,
}

/// 天候状態を管理するリソース
#[derive(Resource)]
pub struct WeatherState {
    pub is_raining: bool,
    pub time_until_change: f32,
}

// ========================================
// Settings Resources
// ========================================

/// カメラ設定を保存するリソース
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct CameraSettings {
    pub mouse_sensitivity: f32,
    pub keyboard_sensitivity: f32,
    pub movement_speed: f32,
    pub zoom_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.003,
            keyboard_sensitivity: 0.02,
            movement_speed: 10.0,
            zoom_speed: 50.0,
        }
    }
}

impl CameraSettings {
    /// 設定ファイルのパスを取得
    pub fn settings_path() -> PathBuf {
        PathBuf::from("assets/user/camera_settings.json")
    }

    /// 設定をファイルに保存
    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Self::settings_path().parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::settings_path(), json)?;
        Ok(())
    }

    /// ファイルから設定を読み込み
    pub fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(Self::settings_path())?;
        let settings = serde_json::from_str(&json)?;
        Ok(settings)
    }

    /// ファイルから設定を読み込むか、デフォルトを使用
    pub fn load_or_default() -> Self {
        Self::load_from_file().unwrap_or_else(|_| {
            println!("設定ファイルが見つかりません。デフォルトを使用します");
            Self::default()
        })
    }
}
