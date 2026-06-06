/// UIボタンを生成するマクロ
///
/// # Example
/// ```
/// spawn_button!(parent, {
///     size: (55.0, 30.0),
///     text: "Move",
///     font_size: 14.0,
///     bg_color: (0.3, 0.5, 0.7),
///     border_color: (0.5, 0.7, 0.9),
///     component: FoxActionButton::Move,
/// });
/// ```
#[macro_export]
macro_rules! spawn_button {
    ($parent:expr, {
        size: ($width:expr, $height:expr),
        text: $text:expr,
        font_size: $font_size:expr,
        bg_color: ($r:expr, $g:expr, $b:expr),
        border_color: ($br:expr, $bg:expr, $bb:expr),
        component: $component:expr $(,)?
    }) => {
        $parent
            .spawn((
                bevy::prelude::Button,
                bevy::prelude::Node {
                    width: bevy::prelude::Val::Px($width),
                    height: bevy::prelude::Val::Px($height),
                    justify_content: bevy::prelude::JustifyContent::Center,
                    align_items: bevy::prelude::AlignItems::Center,
                    border: bevy::prelude::UiRect::all(bevy::prelude::Val::Px(2.0)),
                    ..Default::default()
                },
                bevy::prelude::BackgroundColor(bevy::prelude::Color::srgb($r, $g, $b)),
                bevy::prelude::BorderColor::all(bevy::prelude::Color::srgb($br, $bg, $bb)),
                $component,
            ))
            .with_children(|p| {
                p.spawn((
                    bevy::prelude::Text::new($text),
                    bevy::prelude::TextFont {
                        font_size: $font_size,
                        ..Default::default()
                    },
                    bevy::prelude::TextColor(bevy::prelude::Color::WHITE),
                ));
            })
    };
}

/// 設定行（ラベル + 増減ボタン）を生成するマクロ
#[macro_export]
macro_rules! spawn_setting_row {
    ($parent:expr, {
        label: $label:expr,
        value_type: $value_type:expr,
        down_button: $down:expr,
        up_button: $up:expr $(,)?
    }) => {
        $parent
            .spawn(bevy::prelude::Node {
                flex_direction: bevy::prelude::FlexDirection::Row,
                justify_content: bevy::prelude::JustifyContent::SpaceBetween,
                align_items: bevy::prelude::AlignItems::Center,
                width: bevy::prelude::Val::Percent(100.0),
                column_gap: bevy::prelude::Val::Px(10.0),
                ..Default::default()
            })
            .with_children(|row| {
                row.spawn((
                    bevy::prelude::Text::new($label),
                    bevy::prelude::TextFont {
                        font_size: 20.0,
                        ..Default::default()
                    },
                    bevy::prelude::TextColor(bevy::prelude::Color::WHITE),
                    $value_type,
                ));

                row.spawn(bevy::prelude::Node {
                    flex_direction: bevy::prelude::FlexDirection::Row,
                    column_gap: bevy::prelude::Val::Px(5.0),
                    ..Default::default()
                })
                .with_children(|buttons| {
                    $crate::spawn_button!(buttons, {
                        size: (30.0, 30.0),
                        text: "-",
                        font_size: 20.0,
                        bg_color: (0.4, 0.4, 0.4),
                        border_color: (0.5, 0.5, 0.5),
                        component: $down,
                    });
                    $crate::spawn_button!(buttons, {
                        size: (30.0, 30.0),
                        text: "+",
                        font_size: 20.0,
                        bg_color: (0.4, 0.4, 0.4),
                        border_color: (0.5, 0.5, 0.5),
                        component: $up,
                    });
                });
            });
    };
}

/// アクションボタンのスタイル定義（将来の拡張用）
#[allow(dead_code)]
pub mod button_styles {
    use bevy::prelude::Color;

    /// ボタンスタイルの定義
    #[derive(Clone, Copy)]
    pub struct ButtonStyle {
        pub width: f32,
        pub height: f32,
        pub font_size: f32,
        pub bg_color: Color,
        pub border_color: Color,
    }

    impl ButtonStyle {
        pub const fn new(width: f32, height: f32, font_size: f32, bg: (f32, f32, f32), border: (f32, f32, f32)) -> Self {
            Self {
                width,
                height,
                font_size,
                bg_color: Color::srgb(bg.0, bg.1, bg.2),
                border_color: Color::srgb(border.0, border.1, border.2),
            }
        }
    }

    /// 定義済みボタンスタイル
    pub const MOVE_BUTTON: ButtonStyle = ButtonStyle::new(55.0, 30.0, 14.0, (0.3, 0.5, 0.7), (0.5, 0.7, 0.9));
    pub const BOX_BUTTON: ButtonStyle = ButtonStyle::new(55.0, 30.0, 14.0, (0.5, 0.4, 0.3), (0.7, 0.6, 0.5));
    pub const POSSESSION_BUTTON: ButtonStyle = ButtonStyle::new(80.0, 30.0, 12.0, (0.6, 0.3, 0.6), (0.8, 0.5, 0.8));
    pub const SAVE_BUTTON: ButtonStyle = ButtonStyle::new(120.0, 40.0, 18.0, (0.2, 0.6, 0.2), (0.3, 0.7, 0.3));
    pub const LOAD_BUTTON: ButtonStyle = ButtonStyle::new(120.0, 40.0, 18.0, (0.2, 0.4, 0.7), (0.3, 0.5, 0.8));
    pub const SETTING_BUTTON: ButtonStyle = ButtonStyle::new(30.0, 30.0, 20.0, (0.4, 0.4, 0.4), (0.5, 0.5, 0.5));
}

/// スタイル付きボタンを生成するマクロ
#[macro_export]
macro_rules! spawn_styled_button {
    ($parent:expr, $style:expr, $text:expr, $component:expr) => {
        $parent
            .spawn((
                bevy::prelude::Button,
                bevy::prelude::Node {
                    width: bevy::prelude::Val::Px($style.width),
                    height: bevy::prelude::Val::Px($style.height),
                    justify_content: bevy::prelude::JustifyContent::Center,
                    align_items: bevy::prelude::AlignItems::Center,
                    border: bevy::prelude::UiRect::all(bevy::prelude::Val::Px(2.0)),
                    ..Default::default()
                },
                bevy::prelude::BackgroundColor($style.bg_color),
                bevy::prelude::BorderColor::all($style.border_color),
                $component,
            ))
            .with_children(|p| {
                p.spawn((
                    bevy::prelude::Text::new($text),
                    bevy::prelude::TextFont {
                        font_size: $style.font_size,
                        ..Default::default()
                    },
                    bevy::prelude::TextColor(bevy::prelude::Color::WHITE),
                ));
            })
    };
}
