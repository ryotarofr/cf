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

// Marker component for click feedback text
#[derive(Component)]
struct ClickFeedbackText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, cf_mesh::item_box::setup_item_box))
        .add_systems(
            Update,
            (
                camera_zoom,
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
    // Generate 100x100 field of blocks
    let field_size = 100;
    let block_spacing = 16.0; // Each block is 16x16x16, so we space them by 16 units

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        unlit: true, // Use unlit shading to see texture clearly
        ..default()
    });
    for x in 0..field_size {
        for z in 0..field_size {
            /// TODO: この辺で状態分岐を入れる
            /// Calculate position for each block
            let x_pos = (x as f32 - field_size as f32 / 2.0) * block_spacing;
            let z_pos = (z as f32 - field_size as f32 / 2.0) * block_spacing;

            // Spawn block at calculated position
            commands.spawn((
                Mesh3d(cube_mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_xyz(x_pos, 0.0, z_pos),
                CustomUV,
                Block,
                TextureState { is_special: false },
            ));
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

// System to handle camera zoom with mouse wheel
fn camera_zoom(
    mut wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    for event in wheel_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // Calculate zoom factor based on mouse wheel delta
            let zoom_delta = event.y * 50.0; // Adjust sensitivity as needed

            // Get current distance from origin
            let current_pos = transform.translation;
            let distance = current_pos.length();

            // Calculate new distance (clamped to reasonable range)
            let new_distance = (distance - zoom_delta).clamp(100.0, 3000.0);

            // Apply new position while maintaining direction
            let direction = current_pos.normalize();
            transform.translation = direction * new_distance;
        }
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

    // Check blocks
    for (entity, block_transform, _, _) in block_query.iter() {
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
