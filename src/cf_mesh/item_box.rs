use bevy::prelude::*;

#[derive(Component)]
pub struct ItemBoxButton;

#[derive(Component)]
pub struct ItemBoxPanel;

#[derive(Resource)]
pub struct ItemBoxState {
    pub is_visible: bool,
}

#[derive(Resource)]
pub struct ItemBox {
    pub items: Vec<String>,
}

#[derive(Component)]
pub struct ItemText;

pub fn item_box_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ItemBoxButton>)>,
    mut item_box_state: ResMut<ItemBoxState>,
    mut panel_query: Query<&mut Node, With<ItemBoxPanel>>,
    item_box: Res<ItemBox>,
    mut commands: Commands,
    existing_items_query: Query<Entity, With<ItemText>>,
    panel_entity_query: Query<Entity, With<ItemBoxPanel>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            item_box_state.is_visible = !item_box_state.is_visible;

            if let Ok(mut panel_node) = panel_query.single_mut() {
                panel_node.display = if item_box_state.is_visible {
                    Display::Flex
                } else {
                    Display::None
                };

                if let (true, Ok(panel_entity)) =
                    (item_box_state.is_visible, panel_entity_query.single())
                {
                    for entity in existing_items_query.iter() {
                        commands.entity(entity).despawn();
                    }

                    commands.entity(panel_entity).with_children(|parent| {
                        for item in &item_box.items {
                            parent.spawn((
                                Text::new(format!("• {}", item)),
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                Node {
                                    margin: UiRect::vertical(Val::Px(2.0)),
                                    ..default()
                                },
                                ItemText,
                            ));
                        }
                    });
                }
            }
        }
    }
}

pub fn setup_item_box(mut commands: Commands) {
    // Initialize item box resources
    // memo: is_visible は初表示の状態のこと。 false なら非表示
    // TODO: items は DB から取得。insert_resourceが同期処理なのでロードに時間かかる？
    commands.insert_resource(ItemBoxState { is_visible: false });
    commands.insert_resource(ItemBox {
        items: vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
            "rare_item".to_string(),
        ],
    });

    // Add item box button in top right
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
            ItemBoxButton,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("item box"), TextColor(Color::WHITE)));
        });

    // Add item box panel (initially hidden)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                right: Val::Px(10.0),
                width: Val::Px(200.0),
                height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ItemBoxPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("item box"),
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}
