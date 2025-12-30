use bevy::prelude::*;

// ========================================
// Marker Components
// ========================================

/// カスタムメッシュをマークするためのマーカーコンポーネント
#[derive(Component)]
pub struct CustomUV;

/// メインカメラのマーカーコンポーネント
#[derive(Component)]
pub struct MainCamera;

/// 太陽光のマーカーコンポーネント
#[derive(Component)]
pub struct SunLight;

/// ブロックをマークするコンポーネント
#[derive(Component)]
pub struct Block;

/// 選択可能なブロックをマークするコンポーネント（初期の3x3エリア）
#[derive(Component)]
pub struct Selectable;

/// ブロックのフォーカスハイライトオーバーレイのマーカーコンポーネント
#[derive(Component)]
pub struct BlockHighlight;

/// ブロックが現在ハイライトされているかを追跡するコンポーネント
#[derive(Component)]
pub struct BlockHighlighted;

/// Foxエンティティをマークするコンポーネント
#[derive(Component)]
pub struct Fox;

/// クリックフィードバックテキストのマーカーコンポーネント
#[derive(Component)]
pub struct ClickFeedbackText;

/// アイテムエリアUIのマーカーコンポーネント
#[derive(Component)]
pub struct ItemArea;

/// アイテムスロットのアイコン表示用コンポーネント（画像用）
#[derive(Component)]
pub struct ItemSlotIcon;

/// Foxアクションメニューのマーカーコンポーネント
#[derive(Component)]
pub struct FoxActionMenu;

/// 設定メニューUIのマーカーコンポーネント
#[derive(Component)]
pub struct SettingsMenu;

// ========================================
// Data Components
// ========================================

/// 雨粒をマークするコンポーネント
#[derive(Component)]
pub struct RainDrop {
    pub velocity: Vec3,
    pub lifetime: f32,
}

/// アイテムの種類
#[derive(Clone, Debug)]
pub enum ItemType {
    Fox,
}

/// 個別のアイテムスロットのマーカーコンポーネント
#[derive(Component)]
pub struct ItemSlot {
    pub slot_index: usize,
    pub item: Option<ItemType>,
}

/// Foxアクションボタンの種類
#[derive(Component)]
pub enum FoxActionButton {
    Move,
    Box,
}

/// インタラクティブな設定UIボタンのコンポーネント
#[derive(Component)]
pub enum SettingButton {
    MouseSensitivityUp,
    MouseSensitivityDown,
    KeyboardSensitivityUp,
    KeyboardSensitivityDown,
    MovementSpeedUp,
    MovementSpeedDown,
    ZoomSpeedUp,
    ZoomSpeedDown,
    SaveSettings,
    LoadSettings,
}

/// 設定値を表示するテキストをマークするコンポーネント
#[derive(Component)]
pub enum SettingValueText {
    MouseSensitivity,
    KeyboardSensitivity,
    MovementSpeed,
    ZoomSpeed,
}
