//! Animation Triggers Demo - Shows triggers/windows from map JSON with text display
//!
//! This example demonstrates:
//! - Loading animations from a map project file
//! - Listening to AnimationTriggerEvent and AnimationWindowEvent
//! - Displaying animation events as text in the UI
//!
//! The "tongue" animation has triggers and windows defined in example_project.map.json:
//! - Trigger "show_blurb" at 403ms
//! - Window "enable_hitbox" from 201-701ms
//!
//! Controls:
//! - 1: Play "walk" animation
//! - 2: Play "croak" animation
//! - 3: Play "tongue" animation (has triggers/windows)
//! - 4: Play "hit" animation
//! - Space: Stop animation
//!
//! Run with: cargo run --example animation_triggers_demo -p bevy_map_editor_examples

use bevy::prelude::*;
use bevy_map_animation::{AnimatedSprite, AnimationTriggerEvent, AnimationWindowEvent, WindowPhase, WindowTracker};
use bevy_map_runtime::{AnimatedSpriteHandle, MapRuntimePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Animation Triggers Demo".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MapRuntimePlugin)
        .init_resource::<EventLog>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (handle_events, handle_input, update_display),
        )
        .run();
}

#[derive(Component)]
struct EventDisplay;

#[derive(Resource, Default)]
struct EventLog {
    messages: Vec<String>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // ==========================================================================
    // ONE LINE: Load animated sprite from map project
    // ==========================================================================
    // AnimatedSpriteHandle handles all the loading automatically:
    // - Waits for MapProject to load
    // - Finds sprite sheet by name
    // - Loads the texture
    // - Creates AnimatedSprite and Sprite components
    commands.spawn((
        AnimatedSpriteHandle::new(
            asset_server.load("maps/example_project.map.json"),
            "Frog",
            "tongue", // Start with tongue animation - it has triggers/windows
        )
        .with_scale(4.0),
        WindowTracker::default(), // Required for window events
        Transform::from_xyz(0.0, 50.0, 0.0),
    ));

    // Event log display
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        EventDisplay,
    ));

    info!("Triggers Demo - watch events fire during animations!");
}

/// Handle trigger and window events via MessageReader
fn handle_events(
    mut log: ResMut<EventLog>,
    mut triggers: MessageReader<AnimationTriggerEvent>,
    mut windows: MessageReader<AnimationWindowEvent>,
) {
    for event in triggers.read() {
        log.messages.push(format!(
            "TRIGGER: {} ({})",
            event.trigger_name, event.animation
        ));
    }

    for event in windows.read() {
        // Only log Begin/End, not every Tick
        if event.phase != WindowPhase::Tick {
            log.messages.push(format!(
                "WINDOW: {} {:?} ({})",
                event.window_name, event.phase, event.animation
            ));
        }
    }

    // Keep last 10 messages
    while log.messages.len() > 10 {
        log.messages.remove(0);
    }
}

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut AnimatedSprite>) {
    let animation = if keyboard.just_pressed(KeyCode::Digit1) {
        Some("walk")
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some("croak")
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some("tongue")
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some("hit")
    } else {
        None
    };

    let stop = keyboard.just_pressed(KeyCode::Space);

    if let Ok(mut animated) = query.single_mut() {
        if let Some(name) = animation {
            animated.play(name);
            info!("Playing: {}", name);
        }
        if stop {
            animated.stop();
            info!("Stopped");
        }
    }
}

fn update_display(
    log: Res<EventLog>,
    query: Query<Option<&AnimatedSprite>>,
    mut display_query: Query<&mut Text, With<EventDisplay>>,
) {
    let Ok(mut text) = display_query.single_mut() else {
        return;
    };

    let status = match query.single() {
        Ok(Some(a)) => format!(
            "{} ({})",
            a.current_animation.as_deref().unwrap_or("none"),
            if a.playing { "playing" } else { "stopped" }
        ),
        Ok(None) => "Loading...".to_string(),
        Err(_) => "Not found".to_string(),
    };

    let events_text = if log.messages.is_empty() {
        "No events yet - play 'tongue' animation!".to_string()
    } else {
        log.messages.join("\n")
    };

    *text = Text::new(format!(
        "Animation Triggers Demo\n\n\
        Status: {}\n\n\
        Controls:\n\
        1: walk  2: croak  3: tongue  4: hit\n\
        Space: Stop\n\n\
        Events:\n\
        {}",
        status, events_text
    ));
}
