//! 共通のトレイトとヘルパー関数を定義するモジュール
//!
//! このモジュールは、ゲーム全体で再利用可能な抽象化を提供します：
//! - `RayIntersectable`: レイキャスト判定の統一インターフェース
//! - `Adjustable`: 設定値の増減を抽象化
//! - `Storable`: インベントリアイテムの共通インターフェース
//! - `GameMode`: ゲームモードの状態管理
//! - `CameraRotation`: カメラ回転のヘルパー

#![allow(dead_code)]

use bevy::prelude::*;

// ========================================
// Ray Intersection Trait
// ========================================

/// レイキャスト判定を抽象化するトレイト
///
/// 様々な形状（AABB、球体など）に対してレイとの交差判定を
/// 統一したインターフェースで提供する。
pub trait RayIntersectable {
    /// レイとの交差判定を行い、交差する場合は距離を返す
    ///
    /// # Returns
    /// * `Some(f32)` - 交差する場合、レイの原点から交点までの距離
    /// * `None` - 交差しない場合
    fn ray_intersect(&self, ray: &Ray3d) -> Option<f32>;
}

/// 軸に平行な境界ボックス（AABB）
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    /// 新しいAABBを作成
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self { center, half_extents }
    }

    /// 中心座標とサイズから立方体のAABBを作成
    pub fn cube(center: Vec3, half_size: f32) -> Self {
        Self::new(center, Vec3::splat(half_size))
    }
}

impl RayIntersectable for Aabb {
    fn ray_intersect(&self, ray: &Ray3d) -> Option<f32> {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;

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
}

/// 球体
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl RayIntersectable for Sphere {
    fn ray_intersect(&self, ray: &Ray3d) -> Option<f32> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(*ray.direction);
        let b = 2.0 * oc.dot(*ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t >= 0.0 {
                Some(t)
            } else {
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                if t2 >= 0.0 {
                    Some(t2)
                } else {
                    None
                }
            }
        }
    }
}

// ========================================
// Setting Adjustable Trait
// ========================================

/// 設定値の調整を抽象化するトレイト
///
/// 設定値の増減、範囲制限、表示フォーマットを
/// 統一したインターフェースで提供する。
pub trait Adjustable: Sized {
    /// 値を増加させる
    fn increment(&mut self);
    /// 値を減少させる
    fn decrement(&mut self);
    /// 現在の値を取得
    fn value(&self) -> f32;
    /// 表示用にフォーマット
    fn display(&self) -> String;
    /// ラベル名を取得
    fn label(&self) -> &'static str;
}

/// 調整可能な設定値
#[derive(Clone, Copy, Debug)]
pub struct AdjustableValue {
    pub value: f32,
    pub step: f32,
    pub min: f32,
    pub max: f32,
    pub precision: usize,
    pub label: &'static str,
}

impl AdjustableValue {
    pub const fn new(value: f32, step: f32, min: f32, max: f32, precision: usize, label: &'static str) -> Self {
        Self { value, step, min, max, precision, label }
    }
}

impl Adjustable for AdjustableValue {
    fn increment(&mut self) {
        self.value = (self.value + self.step).min(self.max);
    }

    fn decrement(&mut self) {
        self.value = (self.value - self.step).max(self.min);
    }

    fn value(&self) -> f32 {
        self.value
    }

    fn display(&self) -> String {
        match self.precision {
            1 => format!("{}: {:.1}", self.label, self.value),
            2 => format!("{}: {:.2}", self.label, self.value),
            3 => format!("{}: {:.3}", self.label, self.value),
            _ => format!("{}: {}", self.label, self.value),
        }
    }

    fn label(&self) -> &'static str {
        self.label
    }
}

// ========================================
// Storable Trait (Inventory Items)
// ========================================

/// インベントリに格納可能なアイテムを抽象化するトレイト
pub trait Storable: Clone + std::fmt::Debug + Send + Sync + 'static {
    /// 表示名を取得
    fn display_name(&self) -> &'static str;
    /// アイコンのアセットパスを取得
    fn icon_path(&self) -> &'static str;
}

// ========================================
// Game Mode Trait
// ========================================

/// ゲームモードを抽象化するトレイト
///
/// 各モード（通常、移動、憑依など）の共通インターフェースを定義
pub trait GameMode {
    /// モードがアクティブかどうか
    fn is_active(&self) -> bool;
    /// モードを有効化
    fn activate(&mut self);
    /// モードを無効化
    fn deactivate(&mut self);
    /// カメラ操作が許可されているか
    fn allows_camera_control(&self) -> bool;
    /// WASD入力が許可されているか
    fn allows_wasd_input(&self) -> bool;
}

// ========================================
// Camera Rotation Helpers
// ========================================

/// カメラ回転の結果
#[derive(Clone, Copy, Debug)]
pub struct CameraRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl CameraRotation {
    /// マウスドラッグからカメラ回転を計算
    pub fn from_drag(delta: Vec2, sensitivity: f32, current: &Transform, pitch_limit: f32) -> Self {
        let yaw = -delta.x * sensitivity;
        let pitch = -delta.y * sensitivity;

        let (current_yaw, current_pitch, current_roll) =
            current.rotation.to_euler(bevy::math::EulerRot::YXZ);

        let new_pitch = (current_pitch + pitch).clamp(-pitch_limit, pitch_limit);

        Self {
            yaw: current_yaw + yaw,
            pitch: new_pitch,
            roll: current_roll,
        }
    }

    /// キーボード入力からカメラ回転を計算
    pub fn from_keyboard(
        yaw_delta: f32,
        pitch_delta: f32,
        current: &Transform,
        pitch_limit: f32,
    ) -> Self {
        let (current_yaw, current_pitch, current_roll) =
            current.rotation.to_euler(bevy::math::EulerRot::YXZ);

        let new_pitch = (current_pitch + pitch_delta).clamp(-pitch_limit, pitch_limit);

        Self {
            yaw: current_yaw + yaw_delta,
            pitch: new_pitch,
            roll: current_roll,
        }
    }

    /// Quatに変換
    pub fn to_quat(&self) -> Quat {
        Quat::from_euler(bevy::math::EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
}

/// カメラの向きからXZ平面上の移動方向を計算
pub fn camera_relative_movement(camera_transform: &Transform) -> (Vec3, Vec3) {
    let forward = camera_transform.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = camera_transform.right();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
    (forward_xz, right_xz)
}

// ========================================
// Entity Finding Helpers
// ========================================

/// 子エンティティを再帰的に探索するトレイト
pub trait EntityTreeSearch<'w, 's> {
    /// 条件を満たす子エンティティを探索
    fn find_child<F>(&self, entity: Entity, predicate: F) -> Option<Entity>
    where
        F: Fn(Entity) -> bool + Copy;
}
