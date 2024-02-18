use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::ecs::entity;
use bevy::ecs::query::WorldQuery;
use bevy::pbr::ScreenSpaceAmbientOcclusionBundle;
use bevy::prelude::*;
use bevy::render::mesh::shape::Circle;
use bevy::render::mesh::shape::Cube;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::prelude::*;
use cpal::Stream;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;
use hound::WavWriter;
use serialport::SerialPort;

use cornhacks24_game::recording;
use cornhacks24_game::serial;
use cornhacks24_game::stt;
use cornhacks24_game::req;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
enum AppState {
    #[default]
    MainMenu,
    Options,
    Game,
}

#[derive(Resource)]
struct microphone {
    producer: Sender<i32>
}

#[derive(Event)]
struct StopMic;

#[derive(Event)]
struct StartGame;

#[derive(Resource, Default)]
struct GameInfo {
    game_id: String,
}

#[derive(Resource)]
struct GlobalVars {
    port: Mutex<Box<dyn SerialPort>>,
}

#[derive(Event)]
struct MoveSuspect {
    movement: Movement,
}

enum Movement {
    SendBackActive,
    SendForth(Entity, bevy::prelude::Name),
}

#[derive(Resource, Default)]
struct Settings {
    volume: f32,
    hardware_pg: bool,
    current_device: String,
    hardware_devices: Vec<String>,
}

#[derive(Component)]
struct Away;

#[derive(Component)]
struct Present;

#[derive(Component)]
struct Transitioning;

fn main() {
    //let (port_names, mut port) = serial::setup();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins((
            YarnSpinnerPlugin::new(),
            ExampleYarnSpinnerDialogueViewPlugin::new(),
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TemporalAntiAliasPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(GameInfo::default())
        .add_state::<AppState>()
        .add_event::<MoveSuspect>()
        .add_event::<StartGame>()
        .add_event::<StopMic>()
        .insert_resource(Settings {
            volume: 1.0,
            hardware_pg: true,
            current_device: String::from("jokin"),
            hardware_devices: Vec::new(),
        })
        .insert_resource(Msaa::Off)
        // .insert_resource(GlobalVars {
        //     port: Mutex::new(port),
        // })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, show_options.run_if(in_state(AppState::Options)))
        .add_systems(OnEnter(AppState::Game), launch_game)
        .add_systems(Update, (handle_movements, start_game_listener))
        .add_systems(
            OnEnter(AppState::Game),
            // Spawn the dialogue runner once the Yarn project has finished compiling
            (spawn_dialogue_runner.run_if(resource_added::<YarnProject>())),
        )
        .run();
}

fn handle_keypress (keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        
    }
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let mut dialogue_runner = project.create_dialogue_runner();
    dialogue_runner
        .commands_mut()
        .add_command("send_back_active", send_back_active)
        .add_command("send_forth", send_forth);
    // Immediately start showing the dialogue to the player
    dialogue_runner.start_node("gameplay_loop");
    commands.spawn(dialogue_runner);
}

fn setup_camera(mut commands: Commands) {
    let formation = Transform {
        translation: Vec3::new(-4.6, 3.5, 1.2),
        rotation: Quat::from_euler(EulerRot::XYZ, 0.0, -1.2, 0.0),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    commands
        .spawn((
            Camera3dBundle {
                transform: formation,
                ..default()
            },
            FogSettings {
                color: Color::rgba(0.25, 0.25, 0.25, 1.0),
                falloff: FogFalloff::Linear {
                    start: 5.0,
                    end: 20.0,
                },
                ..default()
            },
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());
}

fn main_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
    let mut ctx = contexts.ctx_mut();

    egui::CentralPanel::default().show(&ctx, |ui| {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(23.0, Proportional)),
            (Body, FontId::new(28.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (
                bevy_egui::egui::TextStyle::Button,
                FontId::new(44.0, Proportional),
            ),
            (Small, FontId::new(20.0, Proportional)),
        ]
        .into();

        ctx.set_style(style);

        ui.vertical_centered(|ui| {
            if ui.add(egui::Button::new("Start")).clicked() {
                next_state.set(AppState::Game);
            }
            if ui.add(egui::Button::new("Options")).clicked() {
                next_state.set(AppState::Options);
            }
            if ui.add(egui::Button::new("Exit")).clicked() {
                std::process::exit(0);
            }
        });
    });
}

fn show_options(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut settings: ResMut<Settings>,
) {
    let mut ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(&ctx, |ui| {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (Heading, FontId::new(50.0, Proportional)),
            (Name("Heading2".into()), FontId::new(35.0, Proportional)),
            (Name("Context".into()), FontId::new(33.0, Proportional)),
            (Body, FontId::new(38.0, Proportional)),
            (Monospace, FontId::new(24.0, Proportional)),
            (
                bevy_egui::egui::TextStyle::Button,
                FontId::new(34.0, Proportional),
            ),
            (Small, FontId::new(30.0, Proportional)),
        ]
        .into();
        
        ctx.set_style(style);
        ui.vertical_centered(|ui| {
            ui.heading("Settings:");
            ui.add(egui::Slider::new(&mut settings.volume, 0.0..=1.0).text("Volume"));
            ui.add(egui::Checkbox::new(
                &mut settings.hardware_pg,
                "Hardware Polygraph",
            ));
            ui.add(egui::Checkbox::new(
                &mut settings.hardware_pg,
                "IDK I CHANGE LATER",
            ));
            if ui.add(egui::Button::new("Main menu")).clicked() {
                next_state.set(AppState::MainMenu);
            }
        });
    });
}

fn launch_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    asset_server: Res<AssetServer>,
    mut game_writer: EventWriter<StartGame>,
    //mut global_vars: ResMut<GlobalVars>,
) {
    game_writer.send(StartGame);

    // Suspects (scary)
    commands.spawn((
        bevy::prelude::Name::new("Clyde"),
        Away,
        PbrBundle {
            mesh: meshes.add(Cube::new(1.0).into()),
            material: materials.add(Color::rgb_u8(255, 0, 0).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        }
    ));
    commands.spawn((
        bevy::prelude::Name::new("Glinda"),
        Away,
        PbrBundle {
            mesh: meshes.add(Cube::new(1.0).into()),
            material: materials.add(Color::rgb_u8(0, 255, 0).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        }
    ));
    commands.spawn((
        bevy::prelude::Name::new("Harry"),
        Away,
        PbrBundle {
            mesh: meshes.add(Cube::new(1.0).into()),
            material: materials.add(Color::rgb_u8(0, 0, 255).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        }
    ));

    commands.spawn(PointLightBundle {
        // transform: Transform::from_xyz(5.0, 8.0, 2.0),
        transform: Transform::from_xyz(1.0, 6.5, 0.0),
        point_light: PointLight {
            intensity: 3000.0, // lumens - roughly a 300W non-halogen incandescent bulb
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Blender scene (poggers)
    commands.spawn((SceneBundle {
        scene: asset_server.load("3d/room.glb#Scene0"),
        transform: Transform::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }),
        ..default()
    },));

    println!("Added assets");
    //recording::record();
    println!("{}", stt::parse_audio());
}

fn start_game_listener(mut game_reader: EventReader<StartGame>) {
    for _ in game_reader.read() {
        let start_response = req::start();
        println!("Game Code: {:?}", start_response);
    }
}

fn send_back_active(In(()): In<()>, mut move_writer: EventWriter<MoveSuspect>) {
    move_writer.send(MoveSuspect { movement: Movement::SendBackActive});
}

fn send_forth(In(suspect_name): In<String>, mut move_writer: EventWriter<MoveSuspect>, query: Query<(&bevy::prelude::Name, Entity), With<Away>>) {
    for suspect in query.iter() {
        if suspect.0.to_string() == suspect_name {
            move_writer.send( MoveSuspect { movement: Movement::SendForth(suspect.1, suspect.0.clone())});
        }
    }
}    

fn handle_movements(
    mut ev_reader: EventReader<MoveSuspect>,
    active: Query<(Entity, &bevy::prelude::Name), With<Present>>,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
) {
    for ev in ev_reader.read() {
        match &ev.movement {
            Movement::SendBackActive => {
                commands.entity(active.single().0).remove::<Present>();
                // Sends Entity back through door
                let mut anim = AnimationClip::default();
                
                anim.add_curve_to_path(EntityPath {parts: vec![active.single().1.clone()]}, VariableCurve {
                    keyframe_timestamps: vec![0.0, 3.0],
                    keyframes: Keyframes::Translation(vec![
                        Vec3::new(2.1, 2.6, 0.2),
                        Vec3::new(4.8, 2.6, -9.5),
                    ]),
                });

                let mut anim_player = AnimationPlayer::default();

                anim_player.play(animations.add(anim));
                commands.entity(active.single().0).insert(anim_player);
            }
            Movement::SendForth(entity, name) => {
                commands.entity(*entity).insert(Present);
                // Sends Entity into chair
                let mut anim = AnimationClip::default();
                
                anim.add_curve_to_path(EntityPath {parts: vec![name.clone()]}, VariableCurve {
                    keyframe_timestamps: vec![0.0, 3.0],
                    keyframes: Keyframes::Translation(vec![
                        Vec3::new(4.8, 2.6, -9.5),
                        Vec3::new(2.1, 2.6, 0.2),
                    ]),
                });

                let mut anim_player = AnimationPlayer::default();

                anim_player.play(animations.add(anim));
                commands.entity(*entity).insert(anim_player);
            }
        }
    }
}