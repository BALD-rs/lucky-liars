use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::prelude::*;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
enum AppState {
    #[default]
    MainMenu,
    Options,
    Game,
}

#[derive(Resource, Default)]
struct Resource {}

#[derive(Resource, Default)]
struct Settings {
    volume: f32,
    hardware_pg: bool,
    hardware_device: String,
}

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
        .insert_resource(Settings::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(Update, show_options.run_if(in_state(AppState::Options)))
        .add_systems(OnEnter(AppState::Game), launch_game)
        .add_systems(
            OnEnter(AppState::Game),
            // Spawn the dialogue runner once the Yarn project has finished compiling
            spawn_dialogue_runner.run_if(resource_added::<YarnProject>()),
        )
        .run();
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let mut dialogue_runner = project.create_dialogue_runner();
    // Immediately start showing the dialogue to the player
    dialogue_runner.start_node("Prologue");
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
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Name("Heading2".into()), FontId::new(25.0, Proportional)),
        (Name("Context".into()), FontId::new(23.0, Proportional)),
        (Body, FontId::new(28.0, Proportional)),
        (Monospace, FontId::new(14.0, Proportional)),
        (bevy_egui::egui::TextStyle::Button, FontId::new(44.0, Proportional)),
        (Small, FontId::new(20.0, Proportional)),
        ].into();

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

fn show_options(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>, mut settings: ResMut<Settings>) {
    let mut ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(&ctx, |ui| {

        let mut style = (*ctx.style()).clone();

        style.text_styles = [
        (Heading, FontId::new(50.0, Proportional)),
        (Name("Heading2".into()), FontId::new(35.0, Proportional)),
        (Name("Context".into()), FontId::new(33.0, Proportional)),
        (Body, FontId::new(38.0, Proportional)),
        (Monospace, FontId::new(24.0, Proportional)),
        (bevy_egui::egui::TextStyle::Button, FontId::new(34.0, Proportional)),
        (Small, FontId::new(30.0, Proportional)),
        ].into();

        ctx.set_style(style);
        ui.vertical_centered(|ui| {
            ui.heading("Settings:");
            ui.add(egui::Slider::new(&mut settings.volume, 0.0..=1.0).text("Volume"));
            ui.add(egui::Checkbox::new(&mut settings.hardware_pg, "Hardware Polygraph"));
            ui.add(egui::Checkbox::new(&mut settings.hardware_pg, "IDK I CHANGE LATER"));
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
