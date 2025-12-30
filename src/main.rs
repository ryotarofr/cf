mod cf_mesh;
mod cf_tool;
mod components;
mod constants;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use plugins::*;
use resources::CameraSettings;
use systems::setup;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraSettings::load_or_default())
        .add_plugins((CameraPlugin, UIPlugin, GameLogicPlugin, WeatherPlugin))
        .add_systems(Startup, setup)
        .run();
}
