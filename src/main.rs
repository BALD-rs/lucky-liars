use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
enum AppState {
    #[default]
    MainMenu,
    Options,
    Game,
}

#[derive(Resource, Default)]
struct Resource {}

#[derive(Component)]
struct Suspect {
    name: String,
}

#[derive(Component)]
struct Away;

#[derive(Component)]
struct Present;

#[derive(Component)]
struct Transitioning;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins((
            // Register the Yarn Spinner plugin using its default settings, which will look for Yarn files in the "dialogue" folder.
            // If this app should support Wasm or Android, we cannot load files without specifying them, so use the following instead.
            // YarnSpinnerPlugin::with_yarn_source(YarnFileSource::file("dialogue/hello_world.yarn")),
            YarnSpinnerPlugin::new(),
            // Initialize the bundled example UI
            ExampleYarnSpinnerDialogueViewPlugin::new(),
        ))
        .add_state::<AppState>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(
            Update,
            // Spawn the dialogue runner once the Yarn project has finished compiling
            spawn_dialogue_runner.run_if(resource_added::<YarnProject>()),
        )
        .add_systems(OnEnter(AppState::Options), show_options)
        .add_systems(OnEnter(AppState::Game), launch_game)
        .run();
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let mut dialogue_runner = project.create_dialogue_runner();
    // Immediately start showing the dialogue to the player
    dialogue_runner.start_node("HelloWorld");
    commands.spawn(dialogue_runner);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn main_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
    let mut ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(&ctx, |ui| {
        if ui.add(egui::Button::new("Options")).clicked() {
            next_state.set(AppState::Options);
        }
        if ui.add(egui::Button::new("Game")).clicked() {
            next_state.set(AppState::Game);
        }
    });
}

fn show_options(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
    let mut ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(&ctx, |ui| {});
}

fn launch_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Suspect {
            name: String::from("Suspect 1"),
        },
        Away,
    ));
    commands.spawn((
        Suspect {
            name: String::from("Suspect 2"),
        },
        Away,
    ));
    commands.spawn((
        Suspect {
            name: String::from("Suspect 3"),
        },
        Away,
    ));
}
