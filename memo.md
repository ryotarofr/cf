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