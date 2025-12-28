mod cf_mesh;
mod cf_tool;

use bevy::{prelude::*, window::PrimaryWindow};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with `With`, they're usually not queried directly since they don't
// contain information within them.
#[derive(Component)]
struct CustomUV;

// Marker component for the main camera
#[derive(Component)]
struct MainCamera;

// Component to mark blocks as selectable
#[derive(Component)]
struct Block;

// Component to mark the selected block
#[derive(Component)]
struct Selected;

// Component to mark blocks as selectable (initial 3x3 area)
#[derive(Component)]
struct Selectable;

// Component to track texture state for individual blocks
#[derive(Component)]
struct TextureState {
    is_special: bool,
}

// Resource to store texture handles
#[derive(Resource)]
struct TextureHandles {
    normal: Handle<Image>,
    special: Handle<Image>,
}

// Resource to store material handles
#[derive(Resource)]
struct BlockMaterials {
    normal: Handle<StandardMaterial>,
    selected: Handle<StandardMaterial>,
}

// Component to mark the Fox entity
#[derive(Component)]
struct Fox;

#[derive(Component)]
struct Tree;

// Marker component for click feedback text
#[derive(Component)]
struct ClickFeedbackText;

// Resource to track mouse drag state
#[derive(Resource, Default)]
struct MouseDragState {
    is_dragging: bool,
    last_position: Option<Vec2>,
}

// Marker component for Fox selection highlight overlay
#[derive(Component)]
struct FoxHighlight;

// Component to track if Fox is currently highlighted
#[derive(Component)]
struct Highlighted;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<MouseDragState>()
        .add_systems(Startup, (setup, cf_mesh::item_box::setup_item_box))
        .add_systems(
            Update,
            (
                camera_zoom,
                camera_drag_rotation,
                camera_keyboard_rotation,
                camera_keyboard_pan,
                fox_hover_highlight,
                block_selection,
                cf_mesh::item_box::item_box_button_system,
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
    let special_texture: Handle<Image> = asset_server.load("special_texture.png");

    /// テスクチャリソースの登録
    ///
    /// `TextureHandles` リソースを通じてアクセス可能
    /// 例:
    /// ```rust
    /// /// special_texture.png を参照する
    /// texture_handles.special.clone()
    /// ```
    /// マテリアルを切り替えるには `BlockMaterials` を使う必要があるので
    /// ほとんどの場合で、`TextureHandles` は `BlockMaterials` と併用する。
    commands.insert_resource(TextureHandles {
        normal: normal_texture.clone(),
        special: special_texture.clone(),
    });

    /// マテリアルとメッシュの初期化
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        unlit: true, // Use unlit shading to see texture clearly
        ..default()
    });
    // Create selected material (highlighted color)
    // Create selected material (highlighted color)
    let selected_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.7, 1.0), // Light blue for selection
        base_color_texture: Some(normal_texture.clone()),
        unlit: true,
        ..default()
    });
    let cube_mesh_handle: Handle<Mesh> = meshes.add(cf_mesh::field::create_cube_mesh());
    let block_materials = BlockMaterials {
        normal: material_handle,
        selected: selected_material_handle,
    };
    /// マテリアルリソースを登録
    commands.insert_resource(block_materials);

    /// フィールドの床部分を作成
    ///
    // Generate 9x9 field of blocks (1 chunk)
    let field_size = 9;
    let block_spacing = 16.0; // Each block is 16x16x16, so we space them by 16 units

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        unlit: true, // Use unlit shading to see texture clearly
        ..default()
    });
    for x in 0..field_size {
        for z in 0..field_size {
            /// Calculate position for each block
            let x_pos = (x as f32 - field_size as f32 / 2.0) * block_spacing;
            let z_pos = (z as f32 - field_size as f32 / 2.0) * block_spacing;

            // Check if this block is in the initial selectable area (3x3 center: indices 3-5, 3-5)
            let is_selectable = (3..=5).contains(&x) && (3..=5).contains(&z);

            // Spawn block at calculated position
            let mut entity_commands = commands.spawn((
                Mesh3d(cube_mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_xyz(x_pos, 0.0, z_pos),
                CustomUV,
                Block,
                TextureState { is_special: false },
            ));

            // Add Selectable component if in the initial 3x3 area
            if is_selectable {
                entity_commands.insert(Selectable);
            }
        }
    }

    // Load and spawn the Fox model on top of the field
    commands.spawn((
        SceneRoot(asset_server.load("animated/Fox.glb#Scene0")),
        Transform::from_xyz(0.0, 8.0, 0.0) // Position at center, on top of blocks (y = 8.0)
            .with_scale(Vec3::splat(0.2)), // Scale to match block size (16x16x16)
        Fox,
        cf_tool::timer::Timer {
            time: 0.0,
            name: "Fox".to_string(),
        },
    ));
    // commands.spawn((
    //     SceneRoot(asset_server.load("animated/tree/TR_01_autumn.glb#Scene0")),
    //     Transform::from_xyz(24.0, 8.0, 0.0) // Position at center, on top of blocks (y = 8.0)
    //         .with_scale(Vec3::splat(20.0)), // Increase scale to make it more visible
    //     Tree,
    //     cf_tool::timer::Timer {
    //         time: 0.0,
    //         name: "Tree".to_string(),
    //     },
    // ));

    // Transform for the camera and lighting, looking at center of the field.
    // Position camera higher and further to see the entire field
    let camera_and_light_transform =
        Transform::from_xyz(1000.0, 1500.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn((Camera3d::default(), camera_and_light_transform, MainCamera));

    // Light up the scene with stronger light for larger area.
    commands.spawn((
        PointLight {
            intensity: 10000000.0,
            range: 5000.0,
            ..default()
        },
        camera_and_light_transform,
    ));

    // Add UI text to display Fox timer
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

    // commands.spawn((
    //     Text::new("Tree Timer: 0.0s"),
    //     Node {
    //         position_type: PositionType::Absolute,
    //         top: Val::Px(20.0),
    //         left: Val::Px(20.0),
    //         ..default()
    //     },
    //     cf_tool::timer::TimerText,
    // ));

    // Add click feedback text
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
}

// System to handle camera zoom with mouse wheel (free camera - move forward/backward)
fn camera_zoom(
    mut wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    for event in wheel_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // Calculate zoom factor based on mouse wheel delta
            let zoom_speed = 50.0; // Adjust sensitivity as needed

            // Move camera forward/backward along its facing direction
            let forward = transform.forward();
            let movement = *forward * event.y * zoom_speed;

            transform.translation += movement;
        }
    }
}

// System to highlight Fox when mouse hovers over it
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
        // Remove highlight when cursor is not in window
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

    // Get ray from camera through cursor position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Check if hovering over Fox
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

    // Manage highlight based on hover state
    let has_highlight = !highlight_query.is_empty();

    if let Some(fox_entity) = hovering_fox_entity {
        if !has_highlight {
            // Add semi-transparent white overlay mesh on top of Fox
            if let Some(pos) = fox_position {
                // Create a simple cube mesh as highlight overlay
                let highlight_material = material_assets.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 0.3), // Semi-transparent white
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                // Spawn a slightly larger cube around the Fox
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(12.0, 12.0, 12.0))),
                    MeshMaterial3d(highlight_material),
                    Transform::from_xyz(pos.x, pos.y, pos.z),
                    FoxHighlight,
                ));
            }
            // Mark Fox as highlighted
            commands.entity(fox_entity).insert(Highlighted);
        }
    } else if has_highlight {
        // Remove highlight
        for highlight_entity in highlight_query.iter() {
            commands.entity(highlight_entity).despawn();
        }
        for fox_entity in highlighted_fox_query.iter() {
            commands.entity(fox_entity).remove::<Highlighted>();
        }
    }
}

// System to handle camera rotation with left mouse button drag (free camera)
fn camera_drag_rotation(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut drag_state: ResMut<MouseDragState>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        drag_state.is_dragging = false;
        drag_state.last_position = None;
        return;
    };

    // Start dragging when left mouse button is pressed
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(last_pos) = drag_state.last_position {
            // Calculate mouse movement delta
            let delta = cursor_position - last_pos;

            if let Ok(mut transform) = camera_query.single_mut() {
                // Rotation sensitivity (reduced for finer control)
                let sensitivity = 0.003;

                // Calculate rotation angles
                let yaw = -delta.x * sensitivity;
                let pitch = -delta.y * sensitivity;

                // Get current rotation as euler angles to clamp pitch
                let (current_yaw, current_pitch, current_roll) =
                    transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

                // Calculate new pitch and clamp it to prevent over-rotation
                let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5); // Limit to ~85 degrees

                // Create new rotation from euler angles
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

// System to handle camera rotation with keyboard (arrow keys only - free camera)
fn camera_keyboard_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // Fine rotation sensitivity for keyboard
    let sensitivity = 0.02;
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    // Horizontal rotation (Left/Right arrows) - Yaw
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        yaw_delta += sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        yaw_delta -= sensitivity;
    }

    // Vertical rotation (Up/Down arrows) - Pitch
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        pitch_delta += sensitivity;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        pitch_delta -= sensitivity;
    }

    // Apply rotation if any keys are pressed
    if yaw_delta != 0.0 || pitch_delta != 0.0 {
        // Get current rotation as euler angles
        let (current_yaw, current_pitch, current_roll) =
            transform.rotation.to_euler(bevy::math::EulerRot::YXZ);

        // Calculate new pitch and clamp it to prevent over-rotation
        let new_pitch = (current_pitch + pitch_delta).clamp(-1.5, 1.5); // Limit to ~85 degrees

        // Create new rotation from euler angles
        transform.rotation = Quat::from_euler(
            bevy::math::EulerRot::YXZ,
            current_yaw + yaw_delta,
            new_pitch,
            current_roll,
        );
    }
}

// System to handle camera panning with WASD keys (free camera movement)
fn camera_keyboard_pan(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // Pan speed (units per frame)
    let pan_speed = 10.0;
    let mut movement = Vec3::ZERO;

    // Get camera's forward and right vectors (projected onto XZ plane for horizontal movement)
    let forward = transform.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = transform.right();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    // W/S keys - move forward/backward relative to camera view
    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += forward_xz * pan_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= forward_xz * pan_speed;
    }

    // A/D keys - move left/right relative to camera view
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= right_xz * pan_speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += right_xz * pan_speed;
    }

    // Apply movement - simply translate the camera position
    if movement != Vec3::ZERO {
        transform.translation += movement;
        // Camera maintains its current rotation direction (no need to look_at)
    }
}

// System to handle block selection with mouse clicks
#[allow(clippy::too_many_arguments)]
fn block_selection(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut block_query: Query<
        (
            Entity,
            &GlobalTransform,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut TextureState,
        ),
        With<Block>,
    >,
    selectable_query: Query<Entity, With<Selectable>>,
    fox_query: Query<(Entity, &GlobalTransform), With<Fox>>,
    mut timer_query: Query<&mut cf_tool::timer::Timer>,
    mut feedback_text_query: Query<&mut Text, With<ClickFeedbackText>>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    materials: Res<BlockMaterials>,
    texture_handles: Res<TextureHandles>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
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

    // Get ray from camera through cursor position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Find the closest object that the ray hits (blocks or fox)
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;

    // Check blocks (only selectable ones)
    for (entity, block_transform, _, _) in block_query.iter() {
        // Only consider selectable blocks
        if !selectable_query.contains(entity) {
            continue;
        }

        // Simple AABB intersection test for cube
        let block_pos = block_transform.translation();
        let half_size = 8.0; // Half of block size (16/2)

        // Check if ray intersects with block's bounding box
        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(half_size))
            .filter(|&distance| distance < closest_distance)
        {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    // Check fox
    for (entity, fox_transform) in fox_query.iter() {
        let fox_pos = fox_transform.translation();
        let fox_half_size = 5.0; // Approximate fox bounding box size

        // Check if ray intersects with fox's bounding box
        if let Some(distance) = ray_box_intersection(&ray, fox_pos, Vec3::splat(fox_half_size))
            .filter(|&distance| distance < closest_distance)
        {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    // Clear previous selection
    for selected_entity in selected_query.iter() {
        commands.entity(selected_entity).remove::<Selected>();
        if let Ok((_, _, mut material, _texture_state)) = block_query.get_mut(selected_entity) {
            material.0 = materials.normal.clone();
        }
    }

    // Apply new selection
    if let Some(selected_entity) = closest_entity {
        commands.entity(selected_entity).insert(Selected);

        // Check if entity has a timer and reset it
        if let Ok(mut timer) = timer_query.get_mut(selected_entity) {
            timer.time = 0.0;
            // Show click feedback
            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = format!("{} clicked! Timer reset!", timer.name);
            }
        } else {
            // Clear feedback if clicking something without a timer
            if let Ok(mut feedback_text) = feedback_text_query.single_mut() {
                feedback_text.0 = "".to_string();
            }
        }

        // Check if it's a block and handle texture switching
        if let Ok((_, _, mut material, mut texture_state)) = block_query.get_mut(selected_entity) {
            // テクスチャの切り替え
            // ブロックごとに状態を持っている。
            texture_state.is_special = !texture_state.is_special;

            // Create new material with appropriate texture
            let new_texture = if texture_state.is_special {
                texture_handles.special.clone()
            } else {
                texture_handles.normal.clone()
            };

            let new_material = material_assets.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.7, 1.0), // Selection color
                base_color_texture: Some(new_texture),
                unlit: true,
                ..default()
            });

            material.0 = new_material;
        }
    }
}

// Helper function for ray-box intersection
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
