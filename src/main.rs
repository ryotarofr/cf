mod cf_mesh;
mod cf_tool;

use bevy::{prelude::*, window::PrimaryWindow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// カスタムメッシュをマークするための「マーカー」コンポーネント
// マーカーコンポーネントはBevyでクエリの`With`フィルタリングによく使われる
// 情報を持たないため、直接クエリされることは少ない
#[derive(Component)]
struct CustomUV;

// メインカメラのマーカーコンポーネント
#[derive(Component)]
struct MainCamera;

// ブロックをマークするコンポーネント
#[derive(Component)]
struct Block;

// 選択可能なブロックをマークするコンポーネント（初期の3x3エリア）
#[derive(Component)]
struct Selectable;

// ブロックのフォーカスハイライトオーバーレイのマーカーコンポーネント
#[derive(Component)]
struct BlockHighlight;

// ブロックが現在ハイライトされているかを追跡するコンポーネント
#[derive(Component)]
struct BlockHighlighted;

// Foxエンティティをマークするコンポーネント
#[derive(Component)]
struct Fox;

// クリックフィードバックテキストのマーカーコンポーネント
#[derive(Component)]
struct ClickFeedbackText;

// アイテムエリアUIのマーカーコンポーネント
#[derive(Component)]
struct ItemArea;

// アイテムの種類
#[derive(Clone, Debug)]
enum ItemType {
    Fox,
}

// 個別のアイテムスロットのマーカーコンポーネント
#[derive(Component)]
struct ItemSlot {
    slot_index: usize,
    item: Option<ItemType>,
}

// アイテムスロットのアイコン表示用コンポーネント
#[derive(Component)]
struct ItemSlotIcon;

// Foxアクションメニューのマーカーコンポーネント
#[derive(Component)]
struct FoxActionMenu;

// Foxアクションボタンの種類
#[derive(Component)]
enum FoxActionButton {
    Move,
    Box,
}

// マウスドラッグの状態を追跡するリソース
#[derive(Resource, Default)]
struct MouseDragState {
    is_dragging: bool,
    last_position: Option<Vec2>,
}

// Fox移動モードの状態を追跡するリソース
#[derive(Resource, Default)]
struct FoxMoveMode {
    is_active: bool,
    is_holding: bool, // キツネを掴んでいるか
    fox_entity: Option<Entity>,
}

// 選択されたアイテムスロットを追跡するリソース
#[derive(Resource, Default)]
struct SelectedItemSlot {
    slot_index: Option<usize>,
    item_type: Option<ItemType>,
}

// Fox選択ハイライトオーバーレイのマーカーコンポーネント
#[derive(Component)]
struct FoxHighlight;

// Foxが現在ハイライトされているかを追跡するコンポーネント
#[derive(Component)]
struct Highlighted;

// 設定メニューの状態を追跡するリソース
#[derive(Resource, Default)]
struct SettingsMenuState {
    is_open: bool,
}

// カメラ設定を保存するリソース
#[derive(Resource, Serialize, Deserialize, Clone)]
struct CameraSettings {
    mouse_sensitivity: f32,
    keyboard_sensitivity: f32,
    movement_speed: f32,
    zoom_speed: f32,
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
    // 設定ファイルのパスを取得
    fn settings_path() -> PathBuf {
        PathBuf::from("assets/user/camera_settings.json")
    }

    // 設定をファイルに保存
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = Self::settings_path().parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::settings_path(), json)?;
        Ok(())
    }

    // ファイルから設定を読み込み
    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(Self::settings_path())?;
        let settings = serde_json::from_str(&json)?;
        Ok(settings)
    }
}

// 設定メニューUIのマーカーコンポーネント
#[derive(Component)]
struct SettingsMenu;

// インタラクティブな設定UIボタンのコンポーネント
#[derive(Component)]
enum SettingButton {
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

// 設定値を表示するテキストをマークするコンポーネント
#[derive(Component)]
enum SettingValueText {
    MouseSensitivity,
    KeyboardSensitivity,
    MovementSpeed,
    ZoomSpeed,
}

// ファイルから設定を読み込むか、デフォルトを使用
fn load_or_default_settings() -> CameraSettings {
    CameraSettings::load_from_file().unwrap_or_else(|_| {
        println!("設定ファイルが見つかりません。デフォルトを使用します");
        CameraSettings::default()
    })
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<MouseDragState>()
        .init_resource::<SettingsMenuState>()
        .init_resource::<FoxMoveMode>()
        .init_resource::<SelectedItemSlot>()
        .insert_resource(load_or_default_settings())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_settings_menu,
                handle_setting_buttons,
                update_setting_value_texts,
                camera_zoom,
                camera_drag_rotation,
                camera_keyboard_rotation,
                camera_keyboard_pan,
                fox_hover_highlight,
                block_hover_highlight,
                block_click_handler,
                handle_fox_action_buttons,
                fox_follow_cursor,
                update_item_slot_display,
                handle_item_slot_click,
                cf_tool::timer::update_timers,
                cf_tool::timer::update_timer_ui,
            ),
        )
        .run();
}

#[allow(unused_doc_comments)]
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    /// テクスチャ画像をロード
    ///
    /// asset_server.load() は定義した段階で非同期で読み込まれる。
    /// 返却値の `Handle<Image>` は即座に利用可能だが、実際のデータが入っているわけではないので
    /// 利用時は、非同期プロセスを待機している。
    let normal_texture: Handle<Image> = asset_server.load("array_texture.png");

    /// マテリアルとメッシュの初期化
    // 選択可能なブロック用のマテリアル（明るい緑色）
    let selectable_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        base_color: Color::srgb(0.8, 1.0, 0.8), // 明るい緑色のティント
        unlit: true,
        ..default()
    });

    // 選択不可能なブロック用のマテリアル（暗い灰色）
    let non_selectable_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        base_color: Color::srgb(0.4, 0.4, 0.4), // 暗い灰色のティント
        unlit: true,
        ..default()
    });

    let cube_mesh_handle: Handle<Mesh> = meshes.add(cf_mesh::field::create_cube_mesh());

    /// フィールドの床部分を作成
    ///
    // 9x9のブロックフィールドを生成（1チャンク）
    let field_size = 9;
    let block_spacing = 16.0; // 各ブロックは16x16x16なので、16ユニット間隔で配置

    for x in 0..field_size {
        for z in 0..field_size {
            /// 各ブロックの位置を計算
            let x_pos = (x as f32 - field_size as f32 / 2.0) * block_spacing;
            let z_pos = (z as f32 - field_size as f32 / 2.0) * block_spacing;

            // このブロックが初期の選択可能エリア（3x3の中央：インデックス3-5, 3-5）にあるかチェック
            let is_selectable = (3..=5).contains(&x) && (3..=5).contains(&z);

            // 選択可能かどうかで異なるマテリアルを使用
            let material = if is_selectable {
                selectable_material_handle.clone()
            } else {
                non_selectable_material_handle.clone()
            };

            // 計算された位置にブロックをスポーン
            let mut entity_commands = commands.spawn((
                Mesh3d(cube_mesh_handle.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(x_pos, 0.0, z_pos),
                CustomUV,
                Block,
            ));

            // 初期の3x3エリアにある場合はSelectableコンポーネントを追加
            if is_selectable {
                entity_commands.insert(Selectable);
            }
        }
    }

    // フィールドの上にFoxモデルを読み込んでスポーン
    commands.spawn((
        SceneRoot(asset_server.load("animated/Fox.glb#Scene0")),
        Transform::from_xyz(0.0, 8.0, 0.0) // 中央、ブロックの上に配置（y = 8.0）
            .with_scale(Vec3::splat(0.2)), // ブロックサイズに合わせてスケール（16x16x16）
        Fox,
        cf_tool::timer::Timer {
            time: 0.0,
            name: "Fox".to_string(),
        },
    ));

    // カメラと照明のトランスフォーム、フィールドの中心を向く
    // カメラをフィールドに近づける
    let camera_and_light_transform =
        Transform::from_xyz(200.0, 300.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y);

    // 3D空間のカメラ
    commands.spawn((Camera3d::default(), camera_and_light_transform, MainCamera));

    // より広いエリアのために強い光でシーンを照らす
    commands.spawn((
        PointLight {
            intensity: 10000000.0,
            range: 5000.0,
            ..default()
        },
        camera_and_light_transform,
    ));

    // Foxタイマーを表示するUIテキストを追加
    commands.spawn((
        Text::new("Fox Timer: 0.0s"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        cf_tool::timer::TimerText,
    ));

    // クリックフィードバックテキストを追加
    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        ClickFeedbackText,
    ));

    // アイテムエリアを画面中央下部に追加
    spawn_item_area(&mut commands);
}

// アイテムエリアUIをスポーンする関数
fn spawn_item_area(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                width: Val::Px(540.0),
                height: Val::Px(60.0),
                // 中央揃えのため、左に半分ずらす
                margin: UiRect {
                    left: Val::Px(-270.0), // 幅の半分
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ItemArea,
        ))
        .with_children(|parent| {
            // 9つのアイテムスロットを作成
            for i in 0..9 {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                        BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                        ItemSlot {
                            slot_index: i,
                            item: None,
                        },
                        Button,
                    ))
                    .with_children(|parent| {
                        // アイテムアイコンを表示（中央）
                        parent.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            ItemSlotIcon,
                        ));
                        // スロット番号を表示
                        parent.spawn((
                            Text::new(format!("{}", i + 1)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(4.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

// マウスホイールでカメラのズームを処理するシステム（フリーカメラ - 前後移動）
fn camera_zoom(
    mut wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
) {
    for event in wheel_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // カメラの向いている方向に沿って前後に移動
            let forward = transform.forward();
            let movement = *forward * event.y * settings.zoom_speed;

            transform.translation += movement;
        }
    }
}

// マウスがFoxの上にホバーしたときにハイライトするシステム
#[allow(clippy::too_many_arguments)]
fn fox_hover_highlight(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    fox_query: Query<(Entity, &GlobalTransform), With<Fox>>,
    mut commands: Commands,
    highlight_query: Query<Entity, With<FoxHighlight>>,
    highlighted_fox_query: Query<Entity, (With<Fox>, With<Highlighted>)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // カーソルがウィンドウ内にない場合はハイライトを削除
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for fox_entity in highlighted_fox_query.iter() {
            commands.entity(fox_entity).remove::<Highlighted>();
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // カメラからカーソル位置を通るレイを取得
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Foxの上にホバーしているかチェック
    let mut hovering_fox_entity = None;
    let mut fox_position = None;

    for (entity, fox_transform) in fox_query.iter() {
        let fox_pos = fox_transform.translation();
        let fox_half_size = 5.0;

        if ray_box_intersection(&ray, fox_pos, Vec3::splat(fox_half_size)).is_some() {
            hovering_fox_entity = Some(entity);
            fox_position = Some(fox_pos);
            break;
        }
    }

    // ホバー状態に基づいてハイライトを管理
    let has_highlight = !highlight_query.is_empty();

    if let Some(fox_entity) = hovering_fox_entity {
        if !has_highlight {
            // Foxの上に半透明の白いオーバーレイメッシュを追加
            if let Some(pos) = fox_position {
                // ハイライトオーバーレイとしてシンプルなキューブメッシュを作成
                let highlight_material = material_assets.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 0.3), // 半透明の白
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                // Foxの周りに少し大きめのキューブをスポーン
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(12.0, 12.0, 12.0))),
                    MeshMaterial3d(highlight_material),
                    Transform::from_xyz(pos.x, pos.y, pos.z),
                    FoxHighlight,
                ));
            }
            // Foxをハイライト済みとしてマーク
            commands.entity(fox_entity).insert(Highlighted);
        }
    } else if has_highlight {
        // ハイライトを削除
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for fox_entity in highlighted_fox_query.iter() {
            commands.entity(fox_entity).remove::<Highlighted>();
        }
    }
}

// 左マウスボタンドラッグでカメラ回転を処理するシステム（フリーカメラ）
fn camera_drag_rotation(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut drag_state: ResMut<MouseDragState>,
    settings: Res<CameraSettings>,
    move_mode: Res<FoxMoveMode>,
) {
    // 移動モード中はカメラドラッグを無効化
    if move_mode.is_active {
        drag_state.is_dragging = false;
        drag_state.last_position = None;
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        drag_state.is_dragging = false;
        drag_state.last_position = None;
        return;
    };

    // 左マウスボタンが押されたときにドラッグを開始
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(last_pos) = drag_state.last_position {
            // マウス移動量を計算
            let delta = cursor_position - last_pos;

            if let Ok(mut transform) = camera_query.single_mut() {
                // 回転角度を計算
                let yaw = -delta.x * settings.mouse_sensitivity;
                let pitch = -delta.y * settings.mouse_sensitivity;

                // ピッチをクランプするために現在の回転をオイラー角として取得
                let (current_yaw, current_pitch, current_roll) =
                    transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

                // 新しいピッチを計算し、過度な回転を防ぐためにクランプ
                let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5); // 約85度に制限

                // オイラー角から新しい回転を作成
                transform.rotation = Quat::from_euler(
                    bevy::math::EulerRot::YXZ,
                    current_yaw + yaw,
                    new_pitch,
                    current_roll,
                );
            }
        }

        drag_state.is_dragging = true;
        drag_state.last_position = Some(cursor_position);
    } else {
        drag_state.is_dragging = false;
        drag_state.last_position = None;
    }
}

// キーボードでカメラ回転を処理するシステム（矢印キーのみ - フリーカメラ）
fn camera_keyboard_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    // 水平回転（左右矢印キー） - Yaw
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        yaw_delta += settings.keyboard_sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        yaw_delta -= settings.keyboard_sensitivity;
    }

    // 垂直回転（上下矢印キー） - Pitch
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        pitch_delta += settings.keyboard_sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        pitch_delta -= settings.keyboard_sensitivity;
    }

    // キーが押されていれば回転を適用
    if yaw_delta != 0.0 || pitch_delta != 0.0 {
        // 現在の回転をオイラー角として取得
        let (current_yaw, current_pitch, current_roll) =
            transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

        // 新しいピッチを計算し、過度な回転を防ぐためにクランプ
        let new_pitch = (current_pitch + pitch_delta).clamp(-1.5, 1.5); // 約85度に制限

        // オイラー角から新しい回転を作成
        transform.rotation = Quat::from_euler(
            bevy::math::EulerRot::YXZ,
            current_yaw + yaw_delta,
            new_pitch,
            current_roll,
        );
    }
}

// WASDキーでカメラパンを処理するシステム（フリーカメラ移動）
fn camera_keyboard_pan(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };
    let mut movement = Vec3::ZERO;

    // カメラの前方と右方向のベクトルを取得（水平移動のためXZ平面に投影）
    let forward = transform.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = transform.right();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    // W/Sキー - カメラビューに対して前後に移動
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += forward_xz * settings.movement_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= forward_xz * settings.movement_speed;
    }

    // A/Dキー - カメラビューに対して左右に移動
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= right_xz * settings.movement_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += right_xz * settings.movement_speed;
    }

    // 移動を適用 - カメラの位置を単純に平行移動
    if movement != Vec3::ZERO {
        transform.translation += movement;
        // カメラは現在の回転方向を維持（look_atは不要）
    }
}

// マウスがブロックの上にホバーしたときにハイライトするシステム
#[allow(clippy::too_many_arguments)]
fn block_hover_highlight(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    block_query: Query<(Entity, &GlobalTransform), With<Block>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut commands: Commands,
    highlight_query: Query<Entity, With<BlockHighlight>>,
    highlighted_block_query: Query<Entity, (With<Block>, With<BlockHighlighted>)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    move_mode: Res<FoxMoveMode>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // カーソルがウィンドウ内にない場合はハイライトを削除
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for block_entity in highlighted_block_query.iter() {
            commands.entity(block_entity).remove::<BlockHighlighted>();
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // カメラからカーソル位置を通るレイを取得
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // 選択可能なブロックの上にホバーしているかチェック
    // 境界での安定した選択を確保するため、最も近いブロックを見つける
    let mut hovering_block_entity = None;
    let mut block_position = None;
    let mut closest_distance = f32::MAX;

    for (entity, block_transform) in block_query.iter() {
        // 選択可能なブロックのみを考慮
        if !selectable_query.contains(entity) {
            continue;
        }

        let block_pos = block_transform.translation();
        let half_size = 8.0; // ブロックサイズの半分 (16/2)

        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(half_size)) {
            // 常に最も近いブロックを選択して境界での安定した動作を保証
            if distance < closest_distance {
                closest_distance = distance;
                hovering_block_entity = Some(entity);
                block_position = Some(block_pos);
            }
        }
    }

    // ハイライトの更新が必要かチェック
    let currently_highlighted = highlighted_block_query.iter().next();

    if let Some(new_block_entity) = hovering_block_entity {
        // ハイライトされているブロックが変わったかチェック
        let needs_update = currently_highlighted != Some(new_block_entity);

        if needs_update {
            // 古いハイライトが存在する場合は削除
            for highlight_entity in highlight_query.iter() {
                commands.entity(highlight_entity).despawn();
            }
            for block_entity in highlighted_block_query.iter() {
                commands.entity(block_entity).remove::<BlockHighlighted>();
            }

            // 新しいハイライトを追加
            if let Some(pos) = block_position {
                // 移動モード中は緑色、通常時は白色のハイライト
                let highlight_color = if move_mode.is_active {
                    Color::srgba(0.0, 1.0, 0.0, 0.4) // 半透明の緑
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.3) // 半透明の白
                };

                let highlight_material = material_assets.add(StandardMaterial {
                    base_color: highlight_color,
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                // ブロックの周りに少し大きめのキューブをスポーン
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(17.0, 17.0, 17.0))),
                    MeshMaterial3d(highlight_material),
                    Transform::from_xyz(pos.x, pos.y, pos.z),
                    BlockHighlight,
                ));
            }
            // 新しいブロックをハイライト済みとしてマーク
            commands.entity(new_block_entity).insert(BlockHighlighted);
        }
    } else {
        // ホバーしているブロックがない場合、すべてのハイライトを削除
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for block_entity in highlighted_block_query.iter() {
            commands.entity(block_entity).remove::<BlockHighlighted>();
        }
    }
}

// ブロッククリックを処理するシステム
#[allow(clippy::too_many_arguments)]
fn block_click_handler(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    block_query: Query<(Entity, &GlobalTransform), With<Block>>,
    selectable_query: Query<Entity, With<Selectable>>,
    fox_query: Query<(Entity, &GlobalTransform), With<Fox>>,
    mut timer_query: Query<&mut cf_tool::timer::Timer>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
    mut commands: Commands,
    action_menu_query: Query<Entity, With<FoxActionMenu>>,
    mut move_mode: ResMut<FoxMoveMode>,
    mut fox_transform_query: Query<&mut Transform, With<Fox>>,
    button_interaction_query: Query<&Interaction, With<Button>>,
    mut selected_slot: ResMut<SelectedItemSlot>,
    mut item_slot_query: Query<&mut ItemSlot>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // UIボタンがクリックされている場合は3Dワールドのクリック処理をスキップ
    for interaction in button_interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            return;
        }
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // カメラからカーソル位置を通るレイを取得
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // レイが当たる最も近いオブジェクトを見つける（ブロックまたはFox）
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;
    let mut is_fox = false;
    let mut fox_position = None;

    // ブロックをチェック（選択可能なもののみ）
    for (entity, block_transform) in block_query.iter() {
        // 選択可能なブロックのみを考慮
        if !selectable_query.contains(entity) {
            continue;
        }

        let block_pos = block_transform.translation();
        let half_size = 8.0; // ブロックサイズの半分 (16/2)

        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(half_size))
            .filter(|&distance| distance < closest_distance)
        {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    // Foxをチェック（移動モード中やアイテム選択中はスキップ）
    if !move_mode.is_active && selected_slot.item_type.is_none() {
        for (entity, fox_transform) in fox_query.iter() {
            let fox_pos = fox_transform.translation();
            let fox_half_size = 5.0; // Foxのバウンディングボックスの概算サイズ

            if let Some(distance) = ray_box_intersection(&ray, fox_pos, Vec3::splat(fox_half_size))
                .filter(|&distance| distance < closest_distance)
            {
                closest_distance = distance;
                closest_entity = Some(entity);
                is_fox = true;
                fox_position = Some(fox_pos);
            }
        }
    }

    // エンティティのクリックを処理
    if let Some(clicked_entity) = closest_entity {
        // 選択されたアイテムがある場合、ブロック上に設置
        if let Some(item_type) = &selected_slot.item_type
            && !is_fox
        {
            // ブロックをクリックした場合のみ設置
            // レイキャストで既に選択可能なブロックのみがclosest_entityに入っている
            if let Some(slot_idx) = selected_slot.slot_index {
                // アイテムスロットからアイテムを削除
                for mut slot in item_slot_query.iter_mut() {
                    if slot.slot_index == slot_idx {
                        slot.item = None;
                        break;
                    }
                }

                // アイテムの種類に応じて設置
                match item_type {
                    ItemType::Fox => {
                        // Foxエンティティを取得して表示
                        if let Ok((fox_entity, _)) = fox_query.single()
                            && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
                        {
                            // クリックしたブロックの上に設置
                            if let Ok((_, block_transform)) = block_query.get(clicked_entity) {
                                let block_pos = block_transform.translation();
                                fox_transform.translation =
                                    Vec3::new(block_pos.x, 8.0, block_pos.z);
                            }
                            // Foxを表示
                            commands.entity(fox_entity).insert(Visibility::Visible);

                            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                                feedback_text.0 = "アイテムを設置しました！".to_string();
                            }
                        }
                    }
                }

                // 選択をクリア
                selected_slot.slot_index = None;
                selected_slot.item_type = None;
                return;
            }
        }

        // 移動モード中にキツネを掴んでいる場合
        if move_mode.is_active && move_mode.is_holding {
            // ブロックをクリックした場合のみ処理
            if !is_fox {
                // キツネを設置（選択可能なブロックの位置に）
                // レイキャストで既に選択可能なブロックのみがclosest_entityに入っている
                if let Some(fox_entity) = move_mode.fox_entity
                    && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
                {
                    // クリックしたブロックの上に設置
                    if let Ok((_, block_transform)) = block_query.get(clicked_entity) {
                        let block_pos = block_transform.translation();
                        fox_transform.translation = Vec3::new(block_pos.x, 8.0, block_pos.z);
                    }

                    if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                        feedback_text.0 = "キツネを設置しました！".to_string();
                    }
                }

                // 移動モードを無効化
                move_mode.is_active = false;
                move_mode.is_holding = false;
                move_mode.fox_entity = None;
                return;
            }
        }

        // Foxをクリックした場合（移動モード中でない場合）
        if is_fox && !move_mode.is_active {
            // 既存のアクションメニューを削除
            for menu_entity in action_menu_query.iter() {
                commands.entity(menu_entity).despawn();
            }

            // Foxの上にアクションメニューを表示
            if let Some(pos) = fox_position {
                spawn_fox_action_menu(&mut commands, pos, camera, camera_transform);
            }
        } else if !is_fox && !move_mode.is_active {
            // Foxではないものをクリックした場合はメニューを閉じる
            for menu_entity in action_menu_query.iter() {
                commands.entity(menu_entity).despawn();
            }
        }

        // エンティティがタイマーを持っている場合はリセット（移動モード中でない場合のみ）
        if !move_mode.is_active {
            if let Ok(mut timer) = timer_query.get_mut(clicked_entity) {
                timer.time = 0.0;
                // クリックフィードバックを表示
                if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                    feedback_text.0 = format!("{} clicked! Timer reset!", timer.name);
                }
            } else {
                // タイマーを持たないものをクリックした場合はフィードバックをクリア
                if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                    feedback_text.0 = "".to_string();
                }
            }
        }
    } else {
        // 何もクリックしていない場合（空中や選択不可能なブロックをクリックした場合）
        if move_mode.is_active && move_mode.is_holding {
            // 移動モード中に選択可能なブロック以外をクリックした場合はエラーメッセージ
            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = "選択可能なブロックにのみ設置できます！".to_string();
            }
            // 移動モードは継続（キツネは掴んだまま）
        } else {
            // メニューを閉じる
            for menu_entity in action_menu_query.iter() {
                commands.entity(menu_entity).despawn();
            }
            // 選択されたアイテムをクリア
            if selected_slot.item_type.is_some() {
                selected_slot.slot_index = None;
                selected_slot.item_type = None;
            }
        }
    }
}

// Foxアクションメニューをスポーンする関数
fn spawn_fox_action_menu(
    commands: &mut Commands,
    fox_position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    // 3D空間のFox位置をスクリーン座標に変換
    let Ok(screen_pos) = camera.world_to_viewport(camera_transform, fox_position) else {
        return;
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.x - 60.0), // ボタンの中央に配置
                top: Val::Px(screen_pos.y - 80.0),  // Foxの上に表示
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
            FoxActionMenu,
        ))
        .with_children(|parent| {
            // Moveボタン
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(55.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.7)),
                    BorderColor::all(Color::srgb(0.5, 0.7, 0.9)),
                    FoxActionButton::Move,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Move"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Boxボタン
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(55.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.4, 0.3)),
                    BorderColor::all(Color::srgb(0.7, 0.6, 0.5)),
                    FoxActionButton::Box,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Box"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

// Foxアクションボタンのクリックを処理するシステム
fn handle_fox_action_buttons(
    interaction_query: Query<(&Interaction, &FoxActionButton), Changed<Interaction>>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
    mut move_mode: ResMut<FoxMoveMode>,
    fox_query: Query<Entity, With<Fox>>,
    mut commands: Commands,
    action_menu_query: Query<Entity, With<FoxActionMenu>>,
    mut item_slot_query: Query<&mut ItemSlot>,
) {
    for (interaction, button_type) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut feedback_text) = feedback_text_query.single_mut()
        {
            match button_type {
                FoxActionButton::Move => {
                    // 移動モードを有効化し、キツネを掴む
                    if let Ok(fox_entity) = fox_query.single() {
                        move_mode.is_active = true;
                        move_mode.is_holding = true;
                        move_mode.fox_entity = Some(fox_entity);
                        feedback_text.0 =
                            "移動モード: 移動先をクリックして設置してください".to_string();

                        // アクションメニューを閉じる
                        for menu_entity in action_menu_query.iter() {
                            commands.entity(menu_entity).despawn();
                        }
                    }
                }
                FoxActionButton::Box => {
                    // キツネをアイテムエリアに格納
                    if let Ok(fox_entity) = fox_query.single() {
                        // 空いているアイテムスロットを探す
                        let mut stored = false;
                        for mut slot in item_slot_query.iter_mut() {
                            if slot.item.is_none() {
                                slot.item = Some(ItemType::Fox);
                                // キツネエンティティを非表示にする
                                commands.entity(fox_entity).insert(Visibility::Hidden);
                                feedback_text.0 =
                                    "キツネをアイテムエリアに格納しました！".to_string();
                                stored = true;
                                break;
                            }
                        }

                        if !stored {
                            feedback_text.0 = "アイテムスロットがいっぱいです！".to_string();
                        }

                        // アクションメニューを閉じる
                        for menu_entity in action_menu_query.iter() {
                            commands.entity(menu_entity).despawn();
                        }
                    }
                }
            }
        }
    }
}

// カーソルに追従してキツネを移動させるシステム
fn fox_follow_cursor(
    move_mode: Res<FoxMoveMode>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut fox_transform_query: Query<&mut Transform, With<Fox>>,
) {
    // 移動モードでキツネを掴んでいる場合のみ
    if !move_mode.is_active || !move_mode.is_holding {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // カーソル位置からレイを取得
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // キツネをy=8の平面上でカーソルに追従させる
    // レイとy=8平面の交点を計算
    let plane_y = 8.0;
    let t = (plane_y - ray.origin.y) / ray.direction.y;

    if t > 0.0 {
        let intersection_point = ray.origin + *ray.direction * t;

        // キツネの位置を更新
        if let Some(fox_entity) = move_mode.fox_entity
            && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
        {
            fox_transform.translation.x = intersection_point.x;
            fox_transform.translation.z = intersection_point.z;
            // y座標は8.0に固定（少し浮かせる）
            fox_transform.translation.y = plane_y + 2.0;
        }
    }
}

// レイとボックスの交差判定のヘルパー関数
fn ray_box_intersection(ray: &Ray3d, box_center: Vec3, half_extents: Vec3) -> Option<f32> {
    let min = box_center - half_extents;
    let max = box_center + half_extents;

    let ray_dir = *ray.direction;
    let inv_dir = Vec3::ONE / ray_dir;
    let t_min = (min - ray.origin) * inv_dir;
    let t_max = (max - ray.origin) * inv_dir;

    let t_enter = t_min.min(t_max).max_element();
    let t_exit = t_min.max(t_max).min_element();

    if t_enter <= t_exit && t_exit >= 0.0 {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

// ESCキーで設定メニューを切り替えるシステム
fn toggle_settings_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings_state: ResMut<SettingsMenuState>,
    mut commands: Commands,
    settings_menu_query: Query<Entity, With<SettingsMenu>>,
    current_settings: Res<CameraSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        settings_state.is_open = !settings_state.is_open;

        if settings_state.is_open {
            // 現在の設定で設定メニューUIを作成
            spawn_settings_menu(&mut commands, &current_settings);
        } else {
            // 設定メニューUIを削除
            for entity in settings_menu_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// 設定ボタンのクリックを処理するシステム
fn handle_setting_buttons(
    interaction_query: Query<(&Interaction, &SettingButton), Changed<Interaction>>,
    mut settings: ResMut<CameraSettings>,
) {
    for (interaction, button_type) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button_type {
                SettingButton::MouseSensitivityUp => {
                    settings.mouse_sensitivity = (settings.mouse_sensitivity + 0.001).min(0.02);
                }
                SettingButton::MouseSensitivityDown => {
                    settings.mouse_sensitivity = (settings.mouse_sensitivity - 0.001).max(0.001);
                }
                SettingButton::KeyboardSensitivityUp => {
                    settings.keyboard_sensitivity = (settings.keyboard_sensitivity + 0.01).min(0.1);
                }
                SettingButton::KeyboardSensitivityDown => {
                    settings.keyboard_sensitivity =
                        (settings.keyboard_sensitivity - 0.01).max(0.01);
                }
                SettingButton::MovementSpeedUp => {
                    settings.movement_speed = (settings.movement_speed + 5.0).min(100.0);
                }
                SettingButton::MovementSpeedDown => {
                    settings.movement_speed = (settings.movement_speed - 5.0).max(5.0);
                }
                SettingButton::ZoomSpeedUp => {
                    settings.zoom_speed = (settings.zoom_speed + 10.0).min(200.0);
                }
                SettingButton::ZoomSpeedDown => {
                    settings.zoom_speed = (settings.zoom_speed - 10.0).max(10.0);
                }
                SettingButton::SaveSettings => {
                    if let Err(e) = settings.save_to_file() {
                        eprintln!("Failed to save settings: {}", e);
                    } else {
                        println!("Settings saved successfully!");
                    }
                }
                SettingButton::LoadSettings => {
                    if let Ok(loaded_settings) = CameraSettings::load_from_file() {
                        *settings = loaded_settings;
                        println!("Settings loaded successfully!");
                    } else {
                        eprintln!("Failed to load settings");
                    }
                }
            }
        }
    }
}

// 設定値テキストを更新するシステム
fn update_setting_value_texts(
    mut text_query: Query<(&mut Text, &SettingValueText)>,
    settings: Res<CameraSettings>,
) {
    if !settings.is_changed() {
        return;
    }

    for (mut text, value_type) in text_query.iter_mut() {
        match value_type {
            SettingValueText::MouseSensitivity => {
                text.0 = format!("Mouse Sensitivity: {:.3}", settings.mouse_sensitivity);
            }
            SettingValueText::KeyboardSensitivity => {
                text.0 = format!("Keyboard Sensitivity: {:.2}", settings.keyboard_sensitivity);
            }
            SettingValueText::MovementSpeed => {
                text.0 = format!("Movement Speed: {:.1}", settings.movement_speed);
            }
            SettingValueText::ZoomSpeed => {
                text.0 = format!("Zoom Speed: {:.1}", settings.zoom_speed);
            }
        }
    }
}

// 設定メニューUIをスポーンするヘルパー関数
fn spawn_settings_menu(commands: &mut Commands, settings: &CameraSettings) {
    // フルスクリーンの半透明オーバーレイを作成
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            SettingsMenu,
        ))
        .with_children(|parent| {
            // 設定パネルコンテナ
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    // タイトル
                    parent.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // カメラ設定セクション
                    parent.spawn((
                        Text::new("Camera Settings"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    // マウス感度
                    parent.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(10.0),
                        ..default()
                    }).with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("Mouse Sensitivity: {:.3}", settings.mouse_sensitivity)),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(Color::WHITE),
                            SettingValueText::MouseSensitivity,
                        ));
                        parent.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(5.0),
                            ..default()
                        }).with_children(|parent| {
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MouseSensitivityDown)).with_children(|parent| {
                                parent.spawn((Text::new("-"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                            });
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MouseSensitivityUp)).with_children(|parent| {
                                parent.spawn((Text::new("+"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
                            });
                        });
                    });

                    // 他の設定も同様 - より短いインライン形式を使用
                    parent.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, width: Val::Percent(100.0), column_gap: Val::Px(10.0), ..default() }).with_children(|parent| {
                        parent.spawn((Text::new(format!("Keyboard Sensitivity: {:.2}", settings.keyboard_sensitivity)), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE), SettingValueText::KeyboardSensitivity));
                        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(5.0), ..default() }).with_children(|parent| {
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::KeyboardSensitivityDown)).with_children(|p| { p.spawn((Text::new("-"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::KeyboardSensitivityUp)).with_children(|p| { p.spawn((Text::new("+"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                        });
                    });

                    parent.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, width: Val::Percent(100.0), column_gap: Val::Px(10.0), ..default() }).with_children(|parent| {
                        parent.spawn((Text::new(format!("Movement Speed: {:.1}", settings.movement_speed)), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE), SettingValueText::MovementSpeed));
                        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(5.0), ..default() }).with_children(|parent| {
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MovementSpeedDown)).with_children(|p| { p.spawn((Text::new("-"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MovementSpeedUp)).with_children(|p| { p.spawn((Text::new("+"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                        });
                    });

                    parent.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, width: Val::Percent(100.0), column_gap: Val::Px(10.0), ..default() }).with_children(|parent| {
                        parent.spawn((Text::new(format!("Zoom Speed: {:.1}", settings.zoom_speed)), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE), SettingValueText::ZoomSpeed));
                        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(5.0), ..default() }).with_children(|parent| {
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::ZoomSpeedDown)).with_children(|p| { p.spawn((Text::new("-"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::ZoomSpeedUp)).with_children(|p| { p.spawn((Text::new("+"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                        });
                    });

                    // スペーサー
                    parent.spawn(Node {
                        height: Val::Px(20.0),
                        ..default()
                    });

                    // 保存/読み込みボタン
                    parent.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(10.0),
                        width: Val::Percent(100.0),
                        ..default()
                    }).with_children(|parent| {
                        // 保存ボタン
                        parent.spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                            SettingButton::SaveSettings,
                        )).with_children(|parent| {
                            parent.spawn((
                                Text::new("Save Settings"),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });

                        // 読み込みボタン
                        parent.spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.7)),
                            SettingButton::LoadSettings,
                        )).with_children(|parent| {
                            parent.spawn((
                                Text::new("Load Settings"),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });

                    // スペーサー
                    parent.spawn(Node {
                        height: Val::Px(10.0),
                        ..default()
                    });

                    // 操作方法セクション
                    parent.spawn((
                        Text::new("Controls"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    parent.spawn((
                        Text::new("WASD - Move\nArrows - Rotate\nMouse Drag - Rotate\nWheel - Zoom\nESC - Toggle"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 閉じる指示
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(20.0),
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Press ESC to close"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                            ));
                        });
                });
        });
}

// アイテムスロットの表示を更新するシステム
fn update_item_slot_display(
    slot_query: Query<(&ItemSlot, &Children), Changed<ItemSlot>>,
    mut icon_query: Query<&mut Text, With<ItemSlotIcon>>,
) {
    for (slot, children) in slot_query.iter() {
        // ItemSlotIconを持つ子エンティティを探す
        for child in children.iter() {
            if let Ok(mut text) = icon_query.get_mut(child) {
                // アイテムの種類に応じてアイコンを表示
                text.0 = match &slot.item {
                    Some(ItemType::Fox) => "🦊".to_string(),
                    None => "".to_string(),
                };
                break;
            }
        }
    }
}

// アイテムスロットのクリック用のクエリ型エイリアス
type ItemSlotClickQuery<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static ItemSlot), (Changed<Interaction>, With<Button>)>;

// アイテムスロットのクリックを処理するシステム
fn handle_item_slot_click(
    interaction_query: ItemSlotClickQuery,
    mut selected_slot: ResMut<SelectedItemSlot>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
) {
    for (interaction, slot) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(item_type) = &slot.item
        {
            // アイテムがあるスロットをクリックした場合、選択状態にする
            selected_slot.slot_index = Some(slot.slot_index);
            selected_slot.item_type = Some(item_type.clone());

            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = format!("アイテムを選択しました: {:?}", item_type);
            }
        }
    }
}
