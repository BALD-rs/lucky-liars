use bevy::audio::AudioPlugin;
use bevy::render::mesh::shape::Plane;
use bevy_egui::egui::style::Spacing;
use bevy_egui::egui::RichText;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use cornhacks24_game::req::start;
use cornhacks24_game::tts;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

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
use cornhacks24_game::req;
use cornhacks24_game::serial;
use cornhacks24_game::stt;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
enum AppState {
    #[default]
    MainMenu,
    Options,
    Game,
}

#[derive(Resource)]
struct Microphone {
    producer: Option<Sender<i32>>,
}

#[derive(Event)]
struct StopMic;

#[derive(Event)]
struct StartGame;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct GameInfo {
    game_id: String,
    dossier_files: HashMap<String, String>,
    killer: String,
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
    let (port_names, mut port) = serial::setup();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins((
            YarnSpinnerPlugin::new(),
            ExampleYarnSpinnerDialogueViewPlugin::new(),
        ))
        //.add_plugins(ResourceInspectorPlugin::<GameInfo>::default())
        //.add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TemporalAntiAliasPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.3,
        })
        .insert_resource(Microphone { producer: None })
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
        .insert_resource(GlobalVars {
            port: Mutex::new(port),
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, show_options.run_if(in_state(AppState::Options)))
        .add_systems(OnEnter(AppState::Game), launch_game)
        .add_systems(
            Update,
            (handle_movements, start_game_listener, handle_keypress),
        )
        .add_systems(
            OnEnter(AppState::Game),
            // Spawn the dialogue runner once the Yarn project has finished compiling
            (spawn_dialogue_runner.run_if(resource_added::<YarnProject>())),
        )
        .run();
}

fn handle_keypress(
    keys: Res<Input<KeyCode>>,
    mut mic: ResMut<Microphone>,
    mut globals: ResMut<GlobalVars>,
    game_info: Res<GameInfo>,
    mut dr: Query<&mut DialogueRunner>,
    mut active: Query<&bevy::prelude::Name, With<Present>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let mut buffer: [u8; 1] = [0; 1];
    match globals.port.get_mut().unwrap().read(&mut buffer) {
        Ok(bytes) => {
            println!("{:?}", buffer);
            if bytes == 1 && (buffer == [48] || buffer == [49]) {
                let button = buffer[0] - 48;
                // Button do be pressed
                if button == 1 || button == 0 {
                    println!("We press");
                    // Already exists, kill
                    if let Some(tx) = &mut mic.producer {
                        // Kill is a go.
                        tx.send(5).unwrap();
                        mic.producer = None;
                        // For later optimizations
                        thread::sleep(Duration::from_millis(200));
                        let output = stt::parse_audio();
                        let active_name = active.single_mut().to_string();
                        let our_roll = rand::thread_rng().gen_range(0..20);
                        let req = req::InterrogateRequest {
                            game_id: game_info.game_id.clone(),
                            name: active_name.clone(),
                            message: output,
                            our_roll: our_roll,
                            sus_roll: 0,
                        };
                        globals
                            .port
                            .get_mut()
                            .unwrap()
                            .write(format!("D{:02}", our_roll).as_bytes());
                        let sus_ponse = req::interrogate(req);
                        let sus_ponse_text = sus_ponse.response;
                        let sus_ponse_confidence = sus_ponse.confidence;
                        globals
                            .port
                            .get_mut()
                            .unwrap()
                            .write(format!("P{:02}", sus_ponse_confidence).as_bytes());
                        commands.spawn(AudioBundle {
                            source: asset_server
                                .load(tts::say(sus_ponse_text.clone(), active_name)),
                            ..default()
                        });
                        let mut diag = dr.get_single_mut().unwrap();
                        let variable_storage = diag.variable_storage_mut();
                        variable_storage.set("$responseText".to_string(), sus_ponse_text.into());
                        diag.stop();
                        diag.start_node("interrogate_response");
                        //println!("{sus_ponse}");
                    } else {
                        // Not yet exist, life
                        let (tx, rx) = channel();
                        thread::spawn(move || {
                            recording::record(rx);
                        });
                        mic.producer = Some(tx);
                    }
                }
                println!("{button}");
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    if keys.just_pressed(KeyCode::Space) {}
}

fn end_game(In(guess): In<(String)>, game_info: Res<GameInfo>, mut dr: Query<&mut DialogueRunner>) {
    let mut diag = dr.get_single_mut().unwrap();
    diag.stop();

    if guess == game_info.killer {
        diag.start_node("win");
    } else {
        diag.start_node("lose");
    }
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let mut dialogue_runner = project.create_dialogue_runner();
    dialogue_runner
        .commands_mut()
        .add_command("send_back_active", send_back_active)
        .add_command("send_forth", send_forth)
        .add_command("end_game", end_game)
        .add_command("stop_poly", stop_poly);
    // Immediately start showing the dialogue to the player
    dialogue_runner.start_node("Prologue");
    commands.spawn(dialogue_runner);
}

fn stop_poly(In(()): In<()>, mut globals: ResMut<GlobalVars>) {
    for i in 0..50 {
        globals.port.get_mut().unwrap().write("S".as_bytes());
        thread::sleep(Duration::from_millis(i));
    }
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
            (Heading, FontId::new(60.0, Proportional)),
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

        ui.horizontal_centered(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Lucky Liars").heading());
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
                "Software Polygraph",
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

    //let mut clydterial = ColorMaterial::from(clyde_image);
    commands.spawn((
        bevy::prelude::Name::new("Clyde"),
        Away,
        SceneBundle {
            scene: asset_server.load("3d/clyde.glb#Scene0"),
            transform: Transform::from_xyz(0.0, -1.5, 0.0),
            ..default()
        },
    ));
    commands.spawn((
        bevy::prelude::Name::new("Glinda"),
        Away,
        SceneBundle {
            scene: asset_server.load("3d/glinda.glb#Scene0"),
            transform: Transform::from_xyz(0.0, -1.5, 0.0),
            ..default()
        },
    ));
    commands.spawn((
        bevy::prelude::Name::new("Harry"),
        Away,
        SceneBundle {
            scene: asset_server.load("3d/harry.glb#Scene0"),
            transform: Transform::from_xyz(0.0, -1.5, 0.0),
            ..default()
        },
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
    commands.spawn(
        (SceneBundle {
            scene: asset_server.load("3d/room.glb#Scene0"),
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            ..default()
        }),
    );

    commands.spawn((
        bevy::prelude::Name::new("door"),
        Door,
        SceneBundle {
            scene: asset_server.load("3d/door.glb#Scene0"),
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            ..default()
        },
    ));
}

fn start_game_listener(mut game_reader: EventReader<StartGame>, mut game: ResMut<GameInfo>) {
    for _ in game_reader.read() {
        let start_response = req::start();
        game.game_id = start_response.game_id;
        game.dossier_files
            .insert("clyde".to_string(), start_response.clyde);
        game.dossier_files
            .insert("glinda".to_string(), start_response.glinda);
        game.dossier_files
            .insert("harry".to_string(), start_response.harry);
        game.killer = start_response.killer;
        //println!("Game Code: {:?}", start_response);
    }
}

fn send_back_active(In(()): In<()>, mut move_writer: EventWriter<MoveSuspect>) {
    move_writer.send(MoveSuspect {
        movement: Movement::SendBackActive,
    });
}

#[derive(Component)]
pub struct Door;

fn send_forth(
    In(suspect_name): In<String>,
    mut move_writer: EventWriter<MoveSuspect>,
    query: Query<(&bevy::prelude::Name, Entity), With<Away>>,

    game_info: Res<GameInfo>,
    mut dr: Query<&mut DialogueRunner>,
) {
    for suspect in query.iter() {
        if suspect.0.to_string() == suspect_name {
            move_writer.send(MoveSuspect {
                movement: Movement::SendForth(suspect.1, suspect.0.clone()),
            });
            // Updates active dossier file in yarn spinner
            let dossier_file = game_info
                .dossier_files
                .get(&suspect_name.to_ascii_lowercase())
                .unwrap();

            let mut diag = dr.get_single_mut().unwrap();
            let variable_storage = diag.variable_storage_mut();
            variable_storage.set("$activeDossier".to_string(), dossier_file.clone().into());
            variable_storage.set("$activeSuspect".to_string(), suspect.0.to_string().into());
        }
    }
}

fn handle_movements(
    mut ev_reader: EventReader<MoveSuspect>,
    active: Query<(Entity, &bevy::prelude::Name), With<Present>>,
    door: Query<(Entity, &bevy::prelude::Name), With<Door>>,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
) {
    for ev in ev_reader.read() {
        match &ev.movement {
            Movement::SendBackActive => {
                commands.entity(active.single().0).remove::<Present>();
                // Sends Entity back through door
                let mut anim = AnimationClip::default();

                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![active.single().1.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: vec![0.0, 3.0],
                        keyframes: Keyframes::Translation(vec![
                            Vec3::new(2.1, 1.6, 0.2),
                            Vec3::new(4.8, 1.6, -9.5),
                        ]),
                    },
                );

                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![door.single().1.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: vec![0.0, 1.5, 3.0],
                        keyframes: Keyframes::Rotation(vec![
                            Quat::from_rotation_y(0.0),
                            Quat::from_rotation_y(2.0),
                            Quat::from_rotation_y(0.0),
                        ]),
                    },
                );

                let mut anim_player = AnimationPlayer::default();

                anim_player.play(animations.add(anim.clone()));
                commands.entity(active.single().0).insert(anim_player);

                let mut anim_player2 = AnimationPlayer::default();
                anim_player2.play(animations.add(anim));
                commands.entity(door.single().0).insert(anim_player2);
            }
            Movement::SendForth(entity, name) => {
                commands.entity(*entity).insert(Present);
                // Sends Entity into chair
                let mut anim = AnimationClip::default();

                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![name.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: vec![0.0, 3.0],
                        keyframes: Keyframes::Translation(vec![
                            Vec3::new(4.8, 1.6, -9.5),
                            Vec3::new(2.1, 1.6, 0.2),
                        ]),
                    },
                );

                anim.add_curve_to_path(
                    EntityPath {
                        parts: vec![door.single().1.clone()],
                    },
                    VariableCurve {
                        keyframe_timestamps: vec![0.0, 1.5, 3.0],
                        keyframes: Keyframes::Rotation(vec![
                            Quat::from_rotation_y(0.0),
                            Quat::from_rotation_y(2.0),
                            Quat::from_rotation_y(0.0),
                        ]),
                    },
                );

                let mut anim_player = AnimationPlayer::default();
                let mut anim_player2 = AnimationPlayer::default();

                anim_player.play(animations.add(anim.clone()));
                anim_player2.play(animations.add(anim));

                commands.entity(*entity).insert(anim_player);
                commands.entity(door.single().0).insert(anim_player2);
            }
        }
    }
}
