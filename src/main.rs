mod cf_mesh;
mod cf_systems;
mod cf_tool;
mod components;
mod constants;
mod plugins;
mod resources;

use bevy::prelude::*;
use cf_systems::setup;
use plugins::*;
use resources::CameraSettings;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraSettings::load_or_default())
        .add_plugins((CameraPlugin, UIPlugin, GameLogicPlugin, WeatherPlugin))
        .add_systems(Startup, setup)
        .run();
}
