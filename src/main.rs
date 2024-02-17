use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

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
enum Location {
    Away,
    Transition,
    Present,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_state::<AppState>()
        .add_systems(Update, main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnEnter(AppState::Options), show_options)
        .add_systems(OnEnter(AppState::Game), launch_game)
        .run();
}

fn main_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
    let mut ctx = contexts.ctx_mut();
    egui::CentralPanel::default().show(&ctx, |ui| {});
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
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let sus1 = commands
        .spawn(Suspect {
            name: String::from("Suspect 1"),
        })
        .id();
    let sus2 = commands
        .spawn(Suspect {
            name: String::from("Suspect 2"),
        })
        .id();
    let sus3 = commands
        .spawn(Suspect {
            name: String::from("Suspect 3"),
        })
        .id();
}
