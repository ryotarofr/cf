use bevy::prelude::*;

use crate::cf_tool;
use crate::components::*;
use crate::constants::*;
use crate::resources::*;

/// レイとボックス（AABB）の交差判定を行う。
///
/// 3D 空間上のレイが軸に平行な境界ボックス（AABB）と交差するかを判定し、
/// 交差する場合はレイの原点から交点までの距離を返す。
///
/// # Arguments
///
/// * `ray` - 判定対象のレイ。原点と方向を持つ。
/// * `box_center` - ボックスの中心座標。
/// * `half_extents` - ボックスの各軸方向の半分のサイズ。
///
/// # Returns
///
/// * `Some(f32)` - レイがボックスと交差する場合、レイの原点から交点までの距離。
/// * `None` - レイがボックスと交差しない場合。
pub fn ray_box_intersection(ray: &Ray3d, box_center: Vec3, half_extents: Vec3) -> Option<f32> {
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

/// マウスカーソルがブロックの上にホバーした際にハイライト表示を行う。
///
/// カーソル位置からレイキャストを行い、選択可能なブロックと交差するかを判定する。
/// 交差したブロックに対してハイライト用の半透明キューブを表示し、
/// 移動モードの状態に応じて色を変更する（通常時：白、移動モード時：緑）。
///
/// カーソルが画面外にある場合や、ホバー対象が変わった場合は
/// 既存のハイライトを削除して新しいハイライトを生成する。
///
/// # Arguments
///
/// * `window_query` - プライマリウィンドウの情報を取得するクエリ。
/// * `camera_query` - メインカメラとその座標変換情報を取得するクエリ。
/// * `block_query` - すべてのブロックエンティティとその座標を取得するクエリ。
/// * `selectable_query` - 選択可能なブロックのみを絞り込むクエリ。
/// * `commands` - エンティティの生成・削除を行うコマンドバッファ。
/// * `highlight_query` - 既存のハイライトエンティティを取得するクエリ。
/// * `highlighted_block_query` - 現在ハイライトされているブロックを取得するクエリ。
/// * `material_assets` - マテリアルアセットの管理リソース。
/// * `meshes` - メッシュアセットの管理リソース。
/// * `move_mode` - キツネの移動モード状態を保持するリソース。
#[allow(clippy::too_many_arguments)]
pub fn block_hover_highlight(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
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

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut hovering_block_entity = None;
    let mut block_position = None;
    let mut closest_distance = f32::MAX;

    for (entity, block_transform) in block_query.iter() {
        if !selectable_query.contains(entity) {
            continue;
        }

        let block_pos = block_transform.translation();

        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(BLOCK_HALF_SIZE))
            && distance < closest_distance
        {
            closest_distance = distance;
            hovering_block_entity = Some(entity);
            block_position = Some(block_pos);
        }
    }

    let currently_highlighted = highlighted_block_query.iter().next();

    if let Some(new_block_entity) = hovering_block_entity {
        let needs_update = currently_highlighted != Some(new_block_entity);

        if needs_update {
            for highlight_entity in highlight_query.iter() {
                commands.entity(highlight_entity).despawn();
            }
            for block_entity in highlighted_block_query.iter() {
                commands.entity(block_entity).remove::<BlockHighlighted>();
            }

            if let Some(pos) = block_position {
                let highlight_color = if move_mode.is_active {
                    Color::srgba(
                        HIGHLIGHT_COLOR_MOVE.0,
                        HIGHLIGHT_COLOR_MOVE.1,
                        HIGHLIGHT_COLOR_MOVE.2,
                        HIGHLIGHT_COLOR_MOVE.3,
                    )
                } else {
                    Color::srgba(
                        HIGHLIGHT_COLOR_NORMAL.0,
                        HIGHLIGHT_COLOR_NORMAL.1,
                        HIGHLIGHT_COLOR_NORMAL.2,
                        HIGHLIGHT_COLOR_NORMAL.3,
                    )
                };

                let highlight_material = material_assets.add(StandardMaterial {
                    base_color: highlight_color,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(
                        BLOCK_HIGHLIGHT_SIZE,
                        BLOCK_HIGHLIGHT_SIZE,
                        BLOCK_HIGHLIGHT_SIZE,
                    ))),
                    MeshMaterial3d(highlight_material),
                    Transform::from_xyz(pos.x, pos.y, pos.z),
                    BlockHighlight,
                ));
            }
            commands.entity(new_block_entity).insert(BlockHighlighted);
        }
    } else {
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for block_entity in highlighted_block_query.iter() {
            commands.entity(block_entity).remove::<BlockHighlighted>();
        }
    }
}

/// マウス左クリックによるブロックおよびキツネの操作を処理する。
///
/// このシステムは以下の複数の機能を統合して処理する：
/// 1. アイテムスロットから選択したアイテムをブロックに設置
/// 2. 移動モード中のキツネをブロックに設置
/// 3. キツネをクリックしてアクションメニューを表示
/// 4. ブロックのタイマーをリセット
///
/// クリック対象はレイキャストで判定し、ブロックとキツネの両方を対象とする。
/// UI ボタンがクリックされた場合は処理をスキップする。
///
/// # Arguments
///
/// * `mouse_input` - マウスボタンの入力状態。
/// * `window_query` - プライマリウィンドウの情報を取得するクエリ。
/// * `camera_query` - メインカメラとその座標変換情報を取得するクエリ。
/// * `block_query` - すべてのブロックエンティティとその座標を取得するクエリ。
/// * `selectable_query` - 選択可能なブロックのみを絞り込むクエリ。
/// * `fox_query` - キツネエンティティとその座標を取得するクエリ。
/// * `timer_query` - タイマーコンポーネントを持つエンティティを取得するクエリ。
/// * `feedback_text_query` - フィードバック用のテキスト UI を取得するクエリ。
/// * `commands` - エンティティの生成・削除を行うコマンドバッファ。
/// * `action_menu_query` - キツネのアクションメニュー UI を取得するクエリ。
/// * `move_mode` - キツネの移動モード状態を保持するリソース。
/// * `fox_transform_query` - キツネの座標変換を変更するクエリ。
/// * `button_interaction_query` - UI ボタンのインタラクション状態を取得するクエリ。
/// * `selected_slot` - 現在選択中のアイテムスロット情報を保持するリソース。
/// * `item_slot_query` - すべてのアイテムスロットを取得するクエリ。
#[allow(clippy::too_many_arguments)]
pub fn block_click_handler(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
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

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;
    let mut is_fox = false;
    let mut fox_position = None;

    for (entity, block_transform) in block_query.iter() {
        if !selectable_query.contains(entity) {
            continue;
        }

        let block_pos = block_transform.translation();

        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(BLOCK_HALF_SIZE))
            .filter(|&distance| distance < closest_distance)
        {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    if !move_mode.is_active && selected_slot.item_type.is_none() {
        for (entity, fox_transform) in fox_query.iter() {
            let fox_pos = fox_transform.translation();

            if let Some(distance) = ray_box_intersection(&ray, fox_pos, Vec3::splat(FOX_HALF_SIZE))
                .filter(|&distance| distance < closest_distance)
            {
                closest_distance = distance;
                closest_entity = Some(entity);
                is_fox = true;
                fox_position = Some(fox_pos);
            }
        }
    }

    if let Some(clicked_entity) = closest_entity {
        if let Some(item_type) = &selected_slot.item_type
            && !is_fox
            && let Some(slot_idx) = selected_slot.slot_index
        {
            for mut slot in item_slot_query.iter_mut() {
                if slot.slot_index == slot_idx {
                    slot.item = None;
                    break;
                }
            }

            match item_type {
                ItemType::Fox => {
                    if let Ok((fox_entity, _)) = fox_query.single()
                        && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
                    {
                        if let Ok((_, block_transform)) = block_query.get(clicked_entity) {
                            let block_pos = block_transform.translation();
                            fox_transform.translation =
                                Vec3::new(block_pos.x, FOX_INITIAL_HEIGHT, block_pos.z);
                        }
                        commands.entity(fox_entity).insert(Visibility::Visible);

                        if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                            feedback_text.0 = "アイテムを設置しました！".to_string();
                        }
                    }
                }
            }

            selected_slot.slot_index = None;
            selected_slot.item_type = None;
            return;
        }

        if move_mode.is_active && move_mode.is_holding && !is_fox {
            if let Some(fox_entity) = move_mode.fox_entity
                && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
            {
                if let Ok((_, block_transform)) = block_query.get(clicked_entity) {
                    let block_pos = block_transform.translation();
                    fox_transform.translation =
                        Vec3::new(block_pos.x, FOX_INITIAL_HEIGHT, block_pos.z);
                }

                if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                    feedback_text.0 = "キツネを設置しました！".to_string();
                }
            }

            move_mode.is_active = false;
            move_mode.is_holding = false;
            move_mode.fox_entity = None;
            return;
        }

        if is_fox && !move_mode.is_active {
            for menu_entity in action_menu_query.iter() {
                commands.entity(menu_entity).despawn();
            }

            if let Some(pos) = fox_position {
                spawn_fox_action_menu(&mut commands, pos, camera, camera_transform);
            }
        } else if !is_fox && !move_mode.is_active {
            for menu_entity in action_menu_query.iter() {
                commands.entity(menu_entity).despawn();
            }
        }

        if !move_mode.is_active {
            if let Ok(mut timer) = timer_query.get_mut(clicked_entity) {
                timer.time = 0.0;
                if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                    feedback_text.0 = format!("{} clicked! Timer reset!", timer.name);
                }
            } else if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = "".to_string();
            }
        }
    } else if move_mode.is_active && move_mode.is_holding {
        if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
            feedback_text.0 = "選択可能なブロックにのみ設置できます！".to_string();
        }
    } else {
        for menu_entity in action_menu_query.iter() {
            commands.entity(menu_entity).despawn();
        }
        if selected_slot.item_type.is_some() {
            selected_slot.slot_index = None;
            selected_slot.item_type = None;
        }
    }
}

/// キツネのアクションメニュー UI を生成する。
///
/// キツネがクリックされた際に呼び出され、キツネの3D位置を画面座標に変換して
/// その近くに「Move」と「Box」の2つのボタンを持つメニューを表示する。
///
/// - **Move ボタン**: キツネを移動モードにして、別のブロックに設置可能にする。
/// - **Box ボタン**: キツネをアイテムスロットに格納して非表示にする。
///
/// # Arguments
///
/// * `commands` - UI ノードやボタンなどのエンティティを生成するための [`Commands`]。
/// * `fox_position` - キツネの3D ワールド座標。
/// * `camera` - メニュー表示位置を計算するためのカメラ。
/// * `camera_transform` - カメラのグローバル座標変換。
fn spawn_fox_action_menu(
    commands: &mut Commands,
    fox_position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let Ok(screen_pos) = camera.world_to_viewport(camera_transform, fox_position) else {
        return;
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.x - 60.0),
                top: Val::Px(screen_pos.y - 80.0),
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

            // Possessionボタン
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.3, 0.6)),
                    BorderColor::all(Color::srgb(0.8, 0.5, 0.8)),
                    FoxActionButton::Possession,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Possession"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// キツネのアクションメニューのボタンクリックを処理する。
///
/// アクションメニューの「Move」または「Box」ボタンがクリックされた際の処理を行う。
///
/// - **Move ボタン**: キツネを移動モードに切り替え、カーソルに追従させる。
///   その後、ブロックをクリックすることでキツネを設置できる。
/// - **Box ボタン**: キツネを空いているアイテムスロットに格納し、
///   キツネを非表示にする。スロットが満杯の場合はエラーメッセージを表示。
///
/// ボタンクリック後はアクションメニューを自動的に閉じる。
///
/// # Arguments
///
/// * `interaction_query` - ボタンのインタラクション状態とボタン種類を取得するクエリ。
/// * `feedback_text_query` - フィードバック用のテキスト UI を取得するクエリ。
/// * `move_mode` - キツネの移動モード状態を保持するリソース。
/// * `fox_query` - キツネエンティティを取得するクエリ。
/// * `commands` - エンティティの削除（メニュー閉じる）などを行うコマンドバッファ。
/// * `action_menu_query` - キツネのアクションメニュー UI を取得するクエリ。
/// * `item_slot_query` - すべてのアイテムスロットを取得するクエリ。
pub fn handle_fox_action_buttons(
    interaction_query: Query<(&Interaction, &FoxActionButton), Changed<Interaction>>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
    mut move_mode: ResMut<FoxMoveMode>,
    mut possession_mode: ResMut<crate::resources::PossessionMode>,
    fox_query: Query<Entity, With<Fox>>,
    mut commands: Commands,
    action_menu_query: Query<Entity, With<FoxActionMenu>>,
    mut item_slot_query: Query<&mut ItemSlot>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    for (interaction, button_type) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Ok(mut feedback_text) = feedback_text_query.single_mut()
        {
            match button_type {
                FoxActionButton::Move => {
                    if let Ok(fox_entity) = fox_query.single() {
                        move_mode.is_active = true;
                        move_mode.is_holding = true;
                        move_mode.fox_entity = Some(fox_entity);
                        feedback_text.0 =
                            "移動モード: 移動先をクリックして設置してください".to_string();

                        for menu_entity in action_menu_query.iter() {
                            commands.entity(menu_entity).despawn();
                        }
                    }
                }
                FoxActionButton::Box => {
                    if let Ok(fox_entity) = fox_query.single() {
                        let mut slots: Vec<_> = item_slot_query.iter_mut().collect();
                        slots.sort_by_key(|slot| slot.slot_index);

                        let mut stored = false;
                        for mut slot in slots {
                            if slot.item.is_none() {
                                slot.item = Some(ItemType::Fox);
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

                        for menu_entity in action_menu_query.iter() {
                            commands.entity(menu_entity).despawn();
                        }
                    }
                }
                FoxActionButton::Possession => {
                    if let Ok(fox_entity) = fox_query.single() {
                        // 現在のカメラ位置を保存
                        if let Ok(camera_transform) = camera_query.single() {
                            possession_mode.previous_camera_transform = Some(*camera_transform);
                        }

                        possession_mode.is_active = true;
                        possession_mode.fox_entity = Some(fox_entity);
                        possession_mode.camera_offset = Vec3::new(0.0, 2.0, -3.0);
                        feedback_text.0 =
                            "Possessionモード: WASDキーでキツネを操作できます (Escで解除)".to_string();

                        for menu_entity in action_menu_query.iter() {
                            commands.entity(menu_entity).despawn();
                        }
                    }
                }
            }
        }
    }
}

/// 移動モード中にキツネをカーソル位置に追従させる。
///
/// キツネが移動モード（`move_mode.is_active && move_mode.is_holding`）の場合、
/// カーソル位置からレイキャストを行い、Y 座標が `FOX_INITIAL_HEIGHT` の平面との
/// 交点にキツネを配置する。実際には少し浮かせて表示するため、
/// `FOX_HOVER_HEIGHT` を加算した高さに設定する。
///
/// カーソルが画面外にある場合は処理をスキップする。
///
/// # Arguments
///
/// * `move_mode` - キツネの移動モード状態を保持するリソース。
/// * `window_query` - プライマリウィンドウの情報を取得するクエリ。
/// * `camera_query` - メインカメラとその座標変換情報を取得するクエリ。
/// * `fox_transform_query` - キツネの座標変換を変更するクエリ。
pub fn fox_follow_cursor(
    move_mode: Res<FoxMoveMode>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut fox_transform_query: Query<&mut Transform, With<Fox>>,
) {
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

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let plane_y = FOX_INITIAL_HEIGHT;
    let t = (plane_y - ray.origin.y) / ray.direction.y;

    if t > 0.0 {
        let intersection_point = ray.origin + *ray.direction * t;

        if let Some(fox_entity) = move_mode.fox_entity
            && let Ok(mut fox_transform) = fox_transform_query.get_mut(fox_entity)
        {
            fox_transform.translation.x = intersection_point.x;
            fox_transform.translation.z = intersection_point.z;
            fox_transform.translation.y = plane_y + FOX_HOVER_HEIGHT;
        }
    }
}

/// Escキーで憑依モードを解除するシステム
pub fn exit_possession_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut possession_mode: ResMut<crate::resources::PossessionMode>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
    mut dash_state: ResMut<crate::resources::DashInputState>,
) {
    if !possession_mode.is_active {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        // カメラを元の位置に戻す
        if let Some(previous_transform) = possession_mode.previous_camera_transform {
            if let Ok(mut camera_transform) = camera_query.single_mut() {
                *camera_transform = previous_transform;
            }
        }

        // Possessionモードを解除
        possession_mode.is_active = false;
        possession_mode.fox_entity = None;
        possession_mode.previous_camera_transform = None;

        // ダッシュ状態もリセット
        dash_state.is_dashing = false;
        dash_state.last_tap_time = None;
        dash_state.last_key = None;

        // フィードバックメッセージを表示
        if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
            feedback_text.0 = "Possessionモードを解除しました".to_string();
        }
    }
}

/// Possessionモード時にWASDキーでキツネを移動させるシステム
pub fn fox_possession_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    possession_mode: Res<crate::resources::PossessionMode>,
    mut fox_query: Query<&mut Transform, With<Fox>>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<Fox>)>,
    time: Res<Time>,
    mut dash_state: ResMut<crate::resources::DashInputState>,
) {
    if !possession_mode.is_active {
        return;
    }

    let Some(fox_entity) = possession_mode.fox_entity else {
        return;
    };

    let Ok(mut fox_transform) = fox_query.get_mut(fox_entity) else {
        return;
    };

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let current_time = time.elapsed_secs();

    // ダブルタップ検出
    let movement_keys = [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD];

    for key in movement_keys.iter() {
        if keyboard_input.just_pressed(*key) {
            if let Some(last_time) = dash_state.last_tap_time {
                if let Some(last_key) = dash_state.last_key {
                    // 同じキーが短時間に2回押された場合、ダッシュ開始
                    if last_key == *key && (current_time - last_time) < dash_state.dash_timeout {
                        dash_state.is_dashing = true;
                    }
                }
            }
            dash_state.last_tap_time = Some(current_time);
            dash_state.last_key = Some(*key);
        }
    }

    // いずれかのキーが離されたらダッシュ解除
    if !keyboard_input.pressed(KeyCode::KeyW)
        && !keyboard_input.pressed(KeyCode::KeyS)
        && !keyboard_input.pressed(KeyCode::KeyA)
        && !keyboard_input.pressed(KeyCode::KeyD)
    {
        dash_state.is_dashing = false;
    }

    let mut movement = Vec3::ZERO;
    let base_speed = 15.0;
    let dash_speed = 50.0;
    let movement_speed = if dash_state.is_dashing {
        dash_speed
    } else {
        base_speed
    };

    // カメラの向きを基準にした前方と右方向を計算（Y軸は無視）
    let camera_forward = camera_transform.forward();
    let forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let camera_right = camera_transform.right();
    let right_xz = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize_or_zero();

    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += forward_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= forward_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= right_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += right_xz;
    }

    if movement != Vec3::ZERO {
        movement = movement.normalize() * movement_speed * time.delta_secs();
        fox_transform.translation += movement;

        // キツネを移動方向に向ける（モデルの前後が逆なので180度回転を追加）
        let target_rotation = Transform::IDENTITY
            .looking_to(movement, Vec3::Y)
            .rotation
            .normalize();
        // Y軸周りに180度回転させる
        let correction = Quat::from_rotation_y(std::f32::consts::PI);
        fox_transform.rotation = target_rotation * correction;
    }
}
