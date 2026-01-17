use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::components::{Fox, MainCamera};
use crate::constants::CAMERA_PITCH_LIMIT;
use crate::resources::{CameraSettings, FoxMoveMode, MouseDragState, PossessionMode};

/// マウスホイールでカメラのズームを処理するシステム（フリーカメラ - 前後移動）
pub fn camera_zoom(
    mut wheel_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
    possession_mode: Res<PossessionMode>,
) {
    // Possessionモード中はズームを無効化
    if possession_mode.is_active {
        return;
    }

    for event in wheel_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            let forward = transform.forward();
            let movement = *forward * event.y * settings.zoom_speed;
            transform.translation += movement;
        }
    }
}

/// 左マウスボタンドラッグでカメラ回転を処理するシステム（フリーカメラ）
pub fn camera_drag_rotation(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut drag_state: ResMut<MouseDragState>,
    settings: Res<CameraSettings>,
    move_mode: Res<FoxMoveMode>,
    possession_mode: Res<PossessionMode>,
) {
    // 移動モード中またはPossessionモード中はカメラドラッグを無効化
    if move_mode.is_active || possession_mode.is_active {
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

    if mouse_input.pressed(MouseButton::Left) {
        if let Some(last_pos) = drag_state.last_position {
            let delta = cursor_position - last_pos;

            if let Ok(mut transform) = camera_query.single_mut() {
                let yaw = -delta.x * settings.mouse_sensitivity;
                let pitch = -delta.y * settings.mouse_sensitivity;

                let (current_yaw, current_pitch, current_roll) =
                    transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

                let new_pitch =
                    (current_pitch + pitch).clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);

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

/// キーボードでカメラ回転を処理するシステム（矢印キーのみ - フリーカメラ）
pub fn camera_keyboard_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
    possession_mode: Res<PossessionMode>,
) {
    // Possessionモード中は矢印キーでのカメラ回転を無効化
    if possession_mode.is_active {
        return;
    }

    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        yaw_delta += settings.keyboard_sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        yaw_delta -= settings.keyboard_sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        pitch_delta += settings.keyboard_sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        pitch_delta -= settings.keyboard_sensitivity;
    }

    if yaw_delta != 0.0 || pitch_delta != 0.0 {
        let (current_yaw, current_pitch, current_roll) =
            transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

        let new_pitch =
            (current_pitch + pitch_delta).clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);

        transform.rotation = Quat::from_euler(
            bevy::math::EulerRot::YXZ,
            current_yaw + yaw_delta,
            new_pitch,
            current_roll,
        );
    }
}

/// WASDキーでカメラパンを処理するシステム（フリーカメラ移動）
pub fn camera_keyboard_pan(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    settings: Res<CameraSettings>,
    possession_mode: Res<PossessionMode>,
) {
    // Possessionモード中はフリーカメラ移動を無効化
    if possession_mode.is_active {
        return;
    }

    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let mut movement = Vec3::ZERO;

    let forward = transform.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = transform.right();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += forward_xz * settings.movement_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= forward_xz * settings.movement_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= right_xz * settings.movement_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += right_xz * settings.movement_speed;
    }

    if movement != Vec3::ZERO {
        transform.translation += movement;
    }
}

/// Possessionモード時にカメラをキツネの視点に追従させるシステム
pub fn possession_camera_follow(
    possession_mode: Res<PossessionMode>,
    fox_query: Query<&GlobalTransform, With<Fox>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    if !possession_mode.is_active {
        return;
    }

    let Some(fox_entity) = possession_mode.fox_entity else {
        return;
    };

    let Ok(fox_transform) = fox_query.get(fox_entity) else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let fox_position = fox_transform.translation();

    // カメラの現在の回転を保持したまま、位置のみを更新
    let (current_yaw, _current_pitch, _) = camera_transform
        .rotation
        .to_euler(bevy::math::EulerRot::YXZ);

    // カメラの距離と高さ（キツネ全体が見えるように調整）
    let camera_distance = -40.0; // キツネからの水平距離（後方への距離）
    let camera_height = 40.0; // キツネからの高さ（上方への距離）

    // 水平方向（Yaw）と垂直方向（Pitch）を考慮したオフセットを計算
    let yaw_offset = Vec3::new(
        -current_yaw.sin() * camera_distance,
        0.0,
        -current_yaw.cos() * camera_distance,
    );

    // カメラをキツネの斜め後ろに配置（回転は変更しない）
    camera_transform.translation = fox_position + yaw_offset + Vec3::new(0.0, camera_height, 0.0);
}

/// Possessionモード時のマウスドラッグでカメラ回転を処理するシステム
pub fn possession_camera_rotation(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut drag_state: ResMut<MouseDragState>,
    settings: Res<CameraSettings>,
    possession_mode: Res<PossessionMode>,
) {
    if !possession_mode.is_active {
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

    if mouse_input.pressed(MouseButton::Left) {
        if let Some(last_pos) = drag_state.last_position {
            let delta = cursor_position - last_pos;

            if let Ok(mut transform) = camera_query.single_mut() {
                let yaw = -delta.x * settings.mouse_sensitivity;
                let pitch = -delta.y * settings.mouse_sensitivity;

                let (current_yaw, current_pitch, current_roll) =
                    transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

                let new_pitch =
                    (current_pitch + pitch).clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);

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
