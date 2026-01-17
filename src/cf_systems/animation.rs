use bevy::prelude::*;

use crate::components::Fox;
use crate::resources::PossessionMode;

/// キツネのアニメーションクリップを事前にロードするためのリソース
#[derive(Resource)]
pub struct FoxAnimationClips {
    pub clips: [Handle<AnimationClip>; 3],
}

impl FromWorld for FoxAnimationClips {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            clips: [
                asset_server.load(GltfAssetLabel::Animation(0).from_asset("animated/Fox.glb")),
                asset_server.load(GltfAssetLabel::Animation(1).from_asset("animated/Fox.glb")),
                asset_server.load(GltfAssetLabel::Animation(2).from_asset("animated/Fox.glb")),
            ],
        }
    }
}

/// キツネの現在の移動状態を追跡するコンポーネント
#[derive(Component)]
pub struct FoxAnimationState {
    pub is_moving: bool,
    pub current_animation: usize,
    pub animation_graphs: [Option<(Handle<AnimationGraph>, AnimationNodeIndex)>; 3],
}

impl Default for FoxAnimationState {
    fn default() -> Self {
        Self {
            is_moving: false,
            current_animation: 0,
            animation_graphs: [None, None, None],
        }
    }
}

/// Foxのアニメーションを再生するシステム
pub fn play_fox_animation(
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    fox_query: Query<(Entity, &Transform), With<Fox>>,
    children_query: Query<&Children>,
    mut player_query: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
    animation_clips: Res<Assets<AnimationClip>>,
    fox_animation_clips: Res<FoxAnimationClips>,
    possession_mode: Res<PossessionMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_state_query: Query<&mut FoxAnimationState>,
    dash_state: Res<crate::resources::DashInputState>,
) {
    for (fox_entity, _fox_transform) in fox_query.iter() {
        // Foxエンティティの子孫からAnimationPlayerを持つエンティティを探す
        if let Some(player_entity) =
            find_animation_player(fox_entity, &children_query, &player_query)
            && let Ok((entity, mut player, graph_handle)) = player_query.get_mut(player_entity)
        {
            // アニメーション状態を取得または作成
            let mut anim_state = if let Ok(state) = animation_state_query.get_mut(fox_entity) {
                state
            } else {
                commands.entity(fox_entity).insert(FoxAnimationState::default());
                continue;
            };

            // Possessionモードで移動中かどうかを判定
            let is_moving = possession_mode.is_active
                && (keyboard_input.pressed(KeyCode::KeyW)
                    || keyboard_input.pressed(KeyCode::KeyS)
                    || keyboard_input.pressed(KeyCode::KeyA)
                    || keyboard_input.pressed(KeyCode::KeyD));

            // 使用するアニメーションを決定
            // ダッシュ中: Animation2, 移動中: Animation1, 待機中: Animation0
            let target_animation = if is_moving && dash_state.is_dashing {
                2
            } else if is_moving {
                1
            } else {
                0
            };

            // 必要なアニメーショングラフを取得または作成
            if anim_state.animation_graphs[target_animation].is_none() {
                // 事前にロードされたアニメーションクリップを取得
                let animation_clip = fox_animation_clips.clips[target_animation].clone();

                // アニメーションクリップが読み込まれているかチェック
                if animation_clips.get(&animation_clip).is_some() {
                    // 新しいアニメーショングラフを作成
                    let mut graph = AnimationGraph::new();

                    // グラフにアニメーションを追加
                    let animation_index = graph.add_clip(animation_clip, 1.0, graph.root);

                    // グラフをアセットに追加
                    let graph_handle = graphs.add(graph);

                    // キャッシュに保存
                    anim_state.animation_graphs[target_animation] =
                        Some((graph_handle, animation_index));
                }
            }

            // アニメーションが変わった場合のみ更新
            if anim_state.current_animation != target_animation {
                if let Some((graph_handle, animation_index)) =
                    &anim_state.animation_graphs[target_animation]
                {
                    // AnimationGraphHandleをエンティティに追加または更新
                    commands
                        .entity(entity)
                        .insert(AnimationGraphHandle(graph_handle.clone()));

                    // アニメーションを再生
                    player.play(*animation_index).repeat();

                    // 状態を更新
                    anim_state.is_moving = is_moving;
                    anim_state.current_animation = target_animation;
                }
            }
        }
    }
}

/// 子エンティティを再帰的に探索してAnimationPlayerを持つエンティティを見つける
fn find_animation_player(
    entity: Entity,
    children_query: &Query<&Children>,
    player_query: &Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
) -> Option<Entity> {
    // 現在のエンティティがAnimationPlayerを持っているかチェック
    if player_query.contains(entity) {
        return Some(entity);
    }

    // 子エンティティを再帰的に探索
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if let Some(player) = find_animation_player(child, children_query, player_query) {
                return Some(player);
            }
        }
    }

    None
}
