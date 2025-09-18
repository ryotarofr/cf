use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    window::PrimaryWindow,
};

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_zoom, block_selection))
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
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_cube_mesh());
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

    // Text to describe the controls.
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

    // Find the closest block that the ray hits
    let mut closest_block = None;
    let mut closest_distance = f32::MAX;

    for (entity, block_transform, _, _) in block_query.iter() {
        // Simple AABB intersection test for cube
        let block_pos = block_transform.translation();
        let half_size = 8.0; // Half of block size (16/2)

        // Check if ray intersects with block's bounding box
        if let Some(distance) = ray_box_intersection(&ray, block_pos, Vec3::splat(half_size))
            .filter(|&distance| distance < closest_distance)
        {
            closest_distance = distance;
            closest_block = Some(entity);
        }
    }

    // Clear previous selection
    for selected_entity in selected_query.iter() {
        commands.entity(selected_entity).remove::<Selected>();
        if let Ok((_, _, mut material, _texture_state)) = block_query.get_mut(selected_entity) {
            material.0 = materials.normal.clone();
        }
    }

    // Apply new selection and toggle texture
    if let Some(selected_entity) = closest_block {
        commands.entity(selected_entity).insert(Selected);
        if let Ok((_, _, mut material, mut texture_state)) = block_query.get_mut(selected_entity) {
            // Toggle texture state
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

#[rustfmt::skip]
fn create_cube_mesh() -> Mesh {
    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vec![
            // top (facing towards +y)
            [-8.0, 8.0, -8.0], // vertex with index 0
            [8.0, 8.0, -8.0], // vertex with index 1
            [8.0, 8.0, 8.0], // etc. until 23
            [-8.0, 8.0, 8.0],
            // bottom   (-y)
            [-8.0, -8.0, -8.0],
            [8.0, -8.0, -8.0],
            [8.0, -8.0, 8.0],
            [-8.0, -8.0, 8.0],
            // right    (+x)
            [8.0, -8.0, -8.0],
            [8.0, -8.0, 8.0],
            [8.0, 8.0, 8.0], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [8.0, 8.0, -8.0],
            // left     (-x)
            [-8.0, -8.0, -8.0],
            [-8.0, -8.0, 8.0],
            [-8.0, 8.0, 8.0],
            [-8.0, 8.0, -8.0],
            // back     (+z)
            [-8.0, -8.0, 8.0],
            [-8.0, 8.0, 8.0],
            [8.0, 8.0, 8.0],
            [8.0, -8.0, 8.0],
            // forward  (-z)
            [-8.0, -8.0, -8.0],
            [-8.0, 8.0, -8.0],
            [8.0, 8.0, -8.0],
            [8.0, -8.0, -8.0],
        ],
    )
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.2], [0.0, 0.0], [1.0, 0.0], [1.0, 0.2],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.45], [0.0, 0.25], [1.0, 0.25], [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
        ],
    )
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    )
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    //
    // The first two defined triangles look like this (marked with the vertex indices,
    // and the axis), when looking down at the top (+y) of the cube:
    //   -Z
    //   ^
    // 0---1
    // |  /|
    // | / | -> +X
    // |/  |
    // 3---2
    //
    // The right face's (+x) triangles look like this, seen from the outside of the cube.
    //   +Y
    //   ^
    // 10--11
    // |  /|
    // | / | -> -Z
    // |/  |
    // 9---8
    //
    // The back face's (+z) triangles look like this, seen from the outside of the cube.
    //   +Y
    //   ^
    // 17--18
    // |\  |
    // | \ | -> +X
    // |  \|
    // 16--19
    .with_inserted_indices(Indices::U32(vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y)
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ]))
}
