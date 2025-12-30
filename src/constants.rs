// ========================================
// Field Constants
// ========================================

/// フィールドのサイズ（ブロック数）
pub const FIELD_SIZE: i32 = 9;

/// ブロックのサイズ（各辺の長さ）
pub const BLOCK_SIZE: f32 = 16.0;

/// ブロックの半分のサイズ
pub const BLOCK_HALF_SIZE: f32 = BLOCK_SIZE / 2.0;

/// ブロック間の間隔
pub const BLOCK_SPACING: f32 = BLOCK_SIZE;

/// 選択可能エリアの開始インデックス
pub const SELECTABLE_AREA_START: i32 = 3;

/// 選択可能エリアの終了インデックス
pub const SELECTABLE_AREA_END: i32 = 5;

// ========================================
// Camera Constants
// ========================================

/// カメラの初期位置
pub const CAMERA_INITIAL_POSITION: (f32, f32, f32) = (200.0, 300.0, 200.0);

/// カメラのピッチ制限（ラジアン）
pub const CAMERA_PITCH_LIMIT: f32 = 1.5;

// ========================================
// Entity Positioning Constants
// ========================================

/// Foxの初期高さ
pub const FOX_INITIAL_HEIGHT: f32 = 8.0;

/// Foxのスケール
pub const FOX_SCALE: f32 = 0.2;

/// Foxのバウンディングボックスの半分のサイズ
pub const FOX_HALF_SIZE: f32 = 5.0;

/// キツネを掴んでいる時の追加高さ
pub const FOX_HOVER_HEIGHT: f32 = 2.0;

/// ブロックハイライトのサイズ
pub const BLOCK_HIGHLIGHT_SIZE: f32 = 17.0;

// ========================================
// Lighting Constants
// ========================================

/// 太陽光の明るさ（晴天時）
pub const SUN_ILLUMINANCE_CLEAR: f32 = 30000.0;

/// 太陽光の明るさ（雨天時）
pub const SUN_ILLUMINANCE_RAIN: f32 = 8000.0;

// ========================================
// Weather Constants
// ========================================

/// 雨粒の生成レート（個/秒）
pub const RAIN_SPAWN_RATE: f32 = 150.0;

/// 雨粒の生成高度
pub const RAIN_SPAWN_HEIGHT: f32 = 200.0;

/// 雨粒の落下速度
pub const RAIN_FALL_VELOCITY: f32 = -200.0;

/// 雨粒のライフタイム（秒）
pub const RAIN_LIFETIME: f32 = 5.0;

/// 雨粒のサイズ（半径、高さ）
pub const RAIN_CAPSULE_RADIUS: f32 = 0.1;
pub const RAIN_CAPSULE_HEIGHT: f32 = 2.0;

/// 天候変化の時間範囲（秒）
pub const WEATHER_INITIAL_CHANGE_MIN: f32 = 30.0;
pub const WEATHER_INITIAL_CHANGE_MAX: f32 = 120.0;
pub const WEATHER_RAIN_DURATION_MIN: f32 = 30.0;
pub const WEATHER_RAIN_DURATION_MAX: f32 = 180.0;
pub const WEATHER_CLEAR_DURATION_MIN: f32 = 60.0;
pub const WEATHER_CLEAR_DURATION_MAX: f32 = 300.0;

// ========================================
// UI Constants
// ========================================

/// アイテムスロットの数
pub const ITEM_SLOT_COUNT: usize = 9;

/// アイテムスロットのサイズ
pub const ITEM_SLOT_SIZE: f32 = 50.0;

/// アイテムアイコンのサイズ
pub const ITEM_ICON_SIZE: f32 = 40.0;

/// アイテムエリアの幅
pub const ITEM_AREA_WIDTH: f32 = 540.0;

/// アイテムエリアの高さ
pub const ITEM_AREA_HEIGHT: f32 = 60.0;

// ========================================
// Color Constants
// ========================================

/// 選択可能なブロックの色（ティント）
pub const SELECTABLE_BLOCK_COLOR: (f32, f32, f32) = (0.8, 1.0, 0.8);

/// 選択不可能なブロックの色（ティント）
pub const NON_SELECTABLE_BLOCK_COLOR: (f32, f32, f32) = (0.4, 0.4, 0.4);

/// ハイライト色（通常時）
pub const HIGHLIGHT_COLOR_NORMAL: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 0.3);

/// ハイライト色（移動モード時）
pub const HIGHLIGHT_COLOR_MOVE: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 0.4);

/// 雨粒の色
pub const RAIN_COLOR: (f32, f32, f32, f32) = (0.7, 0.8, 1.0, 0.6);

/// 選択されたスロットのボーダー色
pub const SELECTED_SLOT_BORDER_COLOR: (f32, f32, f32) = (1.0, 0.8, 0.0);

/// 通常のスロットのボーダー色
pub const NORMAL_SLOT_BORDER_COLOR: (f32, f32, f32) = (0.5, 0.5, 0.5);
