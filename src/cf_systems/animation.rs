use bevy::prelude::*;

use crate::components::Fox;

/// Foxのアニメーションを再生するシステム
pub fn play_fox_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    fox_query: Query<Entity, With<Fox>>,
    children_query: Query<&Children>,
    mut player_query: Query<(Entity, &mut AnimationPlayer, Option<&AnimationGraphHandle>)>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    for fox_entity in fox_query.iter() {
        // Foxエンティティの子孫からAnimationPlayerを持つエンティティを探す
        if let Some(player_entity) =
            find_animation_player(fox_entity, &children_query, &player_query)
            && let Ok((entity, mut player, graph_handle)) = player_query.get_mut(player_entity)
        {
            // すでにAnimationGraphHandleが設定されていて、アニメーションが再生中なら何もしない
            if graph_handle.is_some() && !player.all_finished() {
                continue;
            }

            // Survey アニメーション (Animation0) を読み込む
            let animation_clip: Handle<AnimationClip> =
                asset_server.load(GltfAssetLabel::Animation(0).from_asset("animated/Fox.glb"));

            // アニメーションクリップが読み込まれているかチェック
            if !asset_server.is_loaded_with_dependencies(&animation_clip) {
                continue;
            }

            // アニメーションクリップが実際に存在するかチェック
            if animation_clips.get(&animation_clip).is_none() {
                continue;
            }

            // すでにグラフが設定されている場合は再生のみ
            if let Some(existing_graph_handle) = graph_handle {
                if let Some(graph) = graphs.get(&existing_graph_handle.0) {
                    // グラフからアニメーションノードを取得
                    if let Some(animation_index) = graph.nodes().next() {
                        player.play(animation_index).repeat();
                    }
                }
            } else {
                // 新しいアニメーショングラフを作成
                let mut graph = AnimationGraph::new();

                // グラフにアニメーションを追加
                let animation_index = graph.add_clip(animation_clip, 1.0, graph.root);

                // グラフをアセットに追加
                let graph_handle = graphs.add(graph);

                // AnimationGraphHandleをエンティティに追加
                commands
                    .entity(entity)
                    .insert(AnimationGraphHandle(graph_handle));

                // アニメーションを再生（次のフレームで有効になる）
                player.play(animation_index).repeat();
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
