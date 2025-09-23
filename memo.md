## commands

### insert_resource
`commands.insert_resource()`

BevyのECSシステムにグローバルリソースとして登録。
どのシステム関数からでもアクセス可能。

```rust
commands.insert_resource(TextureHandles {
    normal: normal_texture.clone(),
    special: special_texture.clone(),
});

```

### spawn
`commands.spawn()`

新しいエンティティを作成。個々のエンティティを生成するときに使う。
→ 今回はフィールドの床部分はマス毎に独立した状態管理を行いたいのでこれを使う

## .add_plugins()と.insert_resource()の違い：
結論: .add_plugins() は .insert_resource() を包含している。ロジックの複雑さなどで決めるとよい(やってけばわかる)。

.add_plugins()
- 目的: システム群やコンポーネント定義などの機能セットを追加
- 内容: プラグインが持つStartupシステム、Updateシステム、リソース、コンポーネントなどを一括で登録
- 例: DefaultPlugins（レンダリング、入力、ウィンドウ管理など）、GamePlugin（ゲーム固有のロジック）

.insert_resource()
- 目的: 単一のリソース（グローバルデータ）をECSワールドに追加
- 内容: 1つの値やオブジェクトをリソースとして登録
- 例: ClearColor（背景色）、設定データ、共有状態など

簡潔に言うと：
- .add_plugins() = 機能パッケージを追加
- .insert_resource() = データを追加

main.rs:15のClearColorは背景色という単純なデータなので.insert_resource()を使用しています。