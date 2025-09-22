use bevy::prelude::*;

// Generic component to track object's internal timer
#[derive(Component)]
pub struct Timer {
    pub time: f32,
    pub name: String,
}

// System to update all timers
pub fn update_timers(
    time: Res<Time>,
    mut timer_query: Query<&mut Timer>,
) {
    for mut timer in timer_query.iter_mut() {
        timer.time += time.delta_secs();
    }
}

// System to update timer UI
pub fn update_timer_ui(
    timer_query: Query<&Timer>,
    mut text_query: Query<&mut Text, With<TimerText>>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        let mut timer_text = String::new();
        for timer in timer_query.iter() {
            if !timer_text.is_empty() {
                timer_text.push_str(", ");
            }
            timer_text.push_str(&format!("{}: {:.1}s", timer.name, timer.time));
        }
        if timer_text.is_empty() {
            timer_text = "No timers".to_string();
        }
        text.0 = timer_text;
    }
}

// Marker component for UI text
#[derive(Component)]
pub struct TimerText;