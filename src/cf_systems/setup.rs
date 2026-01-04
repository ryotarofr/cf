use bevy::prelude::*;

use crate::cf_mesh;
use crate::cf_tool;
use crate::components::*;
use crate::constants::*;

/// ゲームのセットアップシステム
#[allow(unused_doc_comments)]
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let fox_icon: Handle<Image> = asset_server.load("animated/Fox_img_512x512.png");

    /// テクスチャ画像をロード
    let normal_texture: Handle<Image> = asset_server.load("array_texture.png");

    let selectable_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        base_color: Color::srgb(
            SELECTABLE_BLOCK_COLOR.0,
            SELECTABLE_BLOCK_COLOR.1,
            SELECTABLE_BLOCK_COLOR.2,
        ),
        unlit: true,
        ..default()
    });

    let non_selectable_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(normal_texture.clone()),
        base_color: Color::srgb(
            NON_SELECTABLE_BLOCK_COLOR.0,
            NON_SELECTABLE_BLOCK_COLOR.1,
            NON_SELECTABLE_BLOCK_COLOR.2,
        ),
        unlit: true,
        ..default()
    });

    let cube_mesh_handle: Handle<Mesh> = meshes.add(cf_mesh::field::create_cube_mesh());

    spawn_field(
        &mut commands,
        &cube_mesh_handle,
        &selectable_material_handle,
        &non_selectable_material_handle,
    );

    spawn_fox(&mut commands, &asset_server);
    spawn_rock(&mut commands, &asset_server);
    spawn_camera_and_light(&mut commands);
    spawn_ui(&mut commands, fox_icon);
}

fn spawn_field(
    commands: &mut Commands,
    cube_mesh_handle: &Handle<Mesh>,
    selectable_material: &Handle<StandardMaterial>,
    non_selectable_material: &Handle<StandardMaterial>,
) {
    for x in 0..FIELD_SIZE {
        for z in 0..FIELD_SIZE {
            let x_pos = (x as f32 - FIELD_SIZE as f32 / 2.0) * BLOCK_SPACING;
            let z_pos = (z as f32 - FIELD_SIZE as f32 / 2.0) * BLOCK_SPACING;

            let is_selectable = (SELECTABLE_AREA_START..=SELECTABLE_AREA_END).contains(&x)
                && (SELECTABLE_AREA_START..=SELECTABLE_AREA_END).contains(&z);

            let material = if is_selectable {
                selectable_material.clone()
            } else {
                non_selectable_material.clone()
            };

            let mut entity_commands = commands.spawn((
                Mesh3d(cube_mesh_handle.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(x_pos, 0.0, z_pos),
                CustomUV,
                Block,
            ));

            if is_selectable {
                entity_commands.insert(Selectable);
            }
        }
    }
}

fn spawn_fox(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SceneRoot(asset_server.load("animated/Fox.glb#Scene0")),
        Transform::from_xyz(0.0, FOX_INITIAL_HEIGHT, 0.0).with_scale(Vec3::splat(FOX_SCALE)),
        Fox,
        cf_tool::timer::Timer {
            time: 0.0,
            name: "Fox".to_string(),
        },
    ));
}

fn spawn_rock(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SceneRoot(asset_server.load("animated/rock.glb#Scene0")),
        Transform::from_xyz(32.0, 8.0, -32.0).with_scale(Vec3::splat(20.0)),
    ));
}

fn spawn_camera_and_light(commands: &mut Commands) {
    let camera_and_light_transform = Transform::from_xyz(
        CAMERA_INITIAL_POSITION.0,
        CAMERA_INITIAL_POSITION.1,
        CAMERA_INITIAL_POSITION.2,
    )
    .looking_at(Vec3::ZERO, Vec3::Y);

    commands.spawn((Camera3d::default(), camera_and_light_transform, MainCamera));

    commands.spawn((
        DirectionalLight {
            illuminance: SUN_ILLUMINANCE_CLEAR,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            bevy::math::EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        SunLight,
    ));
}

fn spawn_ui(commands: &mut Commands, fox_icon: Handle<Image>) {
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

    spawn_item_area(commands, fox_icon);
}

/// アイテムエリアの UI を生成する。
///
/// 画面下部中央にアイテムスロットのコンテナノードを作成し、
/// 定数で定義されたスロット数分のボタン付きスロットと、その中に
/// アイコン画像およびスロット番号のテキストを子要素として配置する。
///
/// 通常は [`spawn_ui`] から呼び出され、ゲーム開始時の UI セットアップの一部として
/// 実行されることを想定している。
///
/// # Arguments
///
/// * `commands` - UI ノードやボタンなどのエンティティを生成するための [`Commands`]。
/// * `fox_icon` - 各アイテムスロット内に表示するキツネアイコン画像の [`Handle<Image>`]。
pub fn spawn_item_area(commands: &mut Commands, fox_icon: Handle<Image>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                width: Val::Px(ITEM_AREA_WIDTH),
                height: Val::Px(ITEM_AREA_HEIGHT),
                margin: UiRect {
                    left: Val::Px(-ITEM_AREA_WIDTH / 2.0),
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
            for i in 0..ITEM_SLOT_COUNT {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(ITEM_SLOT_SIZE),
                            height: Val::Px(ITEM_SLOT_SIZE),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                        BorderColor::all(Color::srgb(
                            NORMAL_SLOT_BORDER_COLOR.0,
                            NORMAL_SLOT_BORDER_COLOR.1,
                            NORMAL_SLOT_BORDER_COLOR.2,
                        )),
                        ItemSlot {
                            slot_index: i,
                            item: None,
                        },
                        Button,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            ImageNode {
                                image: fox_icon.clone(),
                                ..default()
                            },
                            Node {
                                width: Val::Px(ITEM_ICON_SIZE),
                                height: Val::Px(ITEM_ICON_SIZE),
                                ..default()
                            },
                            Visibility::Hidden,
                            ItemSlotIcon,
                        ));
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
