use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

/// ESCキーで設定メニューを切り替えるシステム
pub fn toggle_settings_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings_state: ResMut<SettingsMenuState>,
    mut commands: Commands,
    settings_menu_query: Query<Entity, With<SettingsMenu>>,
    current_settings: Res<CameraSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        settings_state.is_open = !settings_state.is_open;

        if settings_state.is_open {
            spawn_settings_menu(&mut commands, &current_settings);
        } else {
            for entity in settings_menu_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// 設定ボタンのクリックを処理するシステム
pub fn handle_setting_buttons(
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

/// 設定値テキストを更新するシステム
pub fn update_setting_value_texts(
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

/// アイテムスロットの表示を更新するシステム
pub fn update_item_slot_display(
    slot_query: Query<(&ItemSlot, &Children), Changed<ItemSlot>>,
    mut icon_query: Query<&mut Visibility, With<ItemSlotIcon>>,
) {
    for (slot, children) in slot_query.iter() {
        for child in children {
            if let Ok(mut visibility) = icon_query.get_mut(*child) {
                *visibility = match &slot.item {
                    Some(ItemType::Fox) => Visibility::Visible,
                    None => Visibility::Hidden,
                };
                break;
            }
        }
    }
}

/// アイテムスロットのハイライト表示を更新するシステム
pub fn update_item_slot_highlight(
    mut slot_query: Query<(&ItemSlot, &mut BorderColor)>,
    selected_slot: Res<SelectedItemSlot>,
) {
    for (slot, mut border_color) in slot_query.iter_mut() {
        let is_selected = selected_slot
            .slot_index
            .map(|idx| idx == slot.slot_index)
            .unwrap_or(false);

        *border_color = if is_selected {
            BorderColor::all(Color::srgb(
                SELECTED_SLOT_BORDER_COLOR.0,
                SELECTED_SLOT_BORDER_COLOR.1,
                SELECTED_SLOT_BORDER_COLOR.2,
            ))
        } else {
            BorderColor::all(Color::srgb(
                NORMAL_SLOT_BORDER_COLOR.0,
                NORMAL_SLOT_BORDER_COLOR.1,
                NORMAL_SLOT_BORDER_COLOR.2,
            ))
        };
    }
}

/// アイテムスロットのクリックを処理するシステム
pub fn handle_item_slot_click(
    interaction_query: Query<(&Interaction, &ItemSlot), (Changed<Interaction>, With<Button>)>,
    mut selected_slot: ResMut<SelectedItemSlot>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
) {
    for (interaction, slot) in interaction_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(item_type) = &slot.item
        {
            selected_slot.slot_index = Some(slot.slot_index);
            selected_slot.item_type = Some(item_type.clone());

            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = format!("アイテムを選択しました: {:?}", item_type);
            }
        }
    }
}

/// 設定メニューUIをスポーンする関数（元のmain.rsから移植、インライン展開版）
fn spawn_settings_menu(commands: &mut Commands, settings: &CameraSettings) {
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

                    // マウス感度設定行
                    parent.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, width: Val::Percent(100.0), column_gap: Val::Px(10.0), ..default() }).with_children(|parent| {
                        parent.spawn((Text::new(format!("Mouse Sensitivity: {:.3}", settings.mouse_sensitivity)), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE), SettingValueText::MouseSensitivity));
                        parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(5.0), ..default() }).with_children(|parent| {
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MouseSensitivityDown)).with_children(|p| { p.spawn((Text::new("-"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                            parent.spawn((Button, Node { width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.4, 0.4, 0.4)), SettingButton::MouseSensitivityUp)).with_children(|p| { p.spawn((Text::new("+"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE))); });
                        });
                    });

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
                        parent.spawn((Button, Node { width: Val::Px(120.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2, 0.6, 0.2)), SettingButton::SaveSettings)).with_children(|parent| {
                            parent.spawn((Text::new("Save Settings"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                        });
                        parent.spawn((Button, Node { width: Val::Px(120.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2, 0.4, 0.7)), SettingButton::LoadSettings)).with_children(|parent| {
                            parent.spawn((Text::new("Load Settings"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                        });
                    });

                    parent.spawn(Node {
                        height: Val::Px(10.0),
                        ..default()
                    });

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
