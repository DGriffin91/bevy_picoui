use bevy::{math::*, prelude::*, sprite::Anchor};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    widgets::toggle_button,
    PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(mut pico: ResMut<Pico>, mut toggle_states: Local<[[bool; 10]; 10]>) {
    let main_box = pico.add(PicoItem {
        y: Val::Percent(50.0),
        x: Val::Percent(50.0),
        width: Val::Vh(70.0),
        height: Val::Vh(50.0),
        anchor: Anchor::Center,
        ..default()
    });

    {
        let _guard = pico.vstack(Val::Percent(0.5), Val::Percent(1.0), false, &main_box);

        for row in &mut toggle_states {
            let lane = pico.add(PicoItem {
                width: Val::Percent(100.0),
                height: Val::Percent(9.0),
                style: ItemStyle {
                    background_gradient: (RGB_PALETTE[1][1] * 0.3, RGB_PALETTE[1][0] * 0.8),
                    ..default()
                },
                anchor: Anchor::TopLeft,
                parent: Some(main_box),
                ..default()
            });
            {
                let _guard = pico.hstack(Val::Percent(0.5), Val::Percent(1.0), false, &lane);
                for toggle_state in row {
                    toggle_button(
                        &mut pico,
                        PicoItem {
                            y: Val::Percent(50.0),
                            width: Val::Percent(9.0),
                            height: Val::Percent(80.0),
                            style: ItemStyle {
                                corner_radius: Val::Percent(50.0),
                                background_gradient: (RGB_PALETTE[1][4], RGB_PALETTE[1][1]),
                                ..default()
                            },
                            anchor: Anchor::CenterLeft,
                            parent: Some(lane),
                            ..default()
                        },
                        // This color will be added to the existing gradient.
                        Color::rgb(0.25, 0.25, 0.25),
                        toggle_state,
                    );
                }
            }
        }
    }

    let top_down_box = pico.add(PicoItem {
        y: Val::Percent(10.0),
        x: Val::Percent(10.0),
        width: Val::Percent(10.0),
        height: Val::Percent(80.0),
        anchor: Anchor::TopLeft,
        style: ItemStyle {
            corner_radius: Val::Vh(4.0),
            background_gradient: (RGB_PALETTE[0][0] * 0.6, RGB_PALETTE[0][0] * 0.1),
            ..default()
        },
        ..default()
    });
    {
        let _guard = pico.vstack(Val::Percent(0.0), Val::Percent(-1.0), false, &top_down_box);
        for row in 0..7 {
            pico.add(PicoItem {
                width: Val::Percent(100.0),
                height: Val::Percent(9.0),
                anchor: Anchor::TopCenter,
                anchor_parent: Anchor::TopCenter,
                style: ItemStyle {
                    corner_radius: Val::Vh(4.0),
                    background_color: RGB_PALETTE[0][row],
                    ..default()
                },
                parent: Some(top_down_box),
                ..default()
            });
        }
    }

    let bottom_up_box = pico.add(PicoItem {
        y: Val::Percent(10.0),
        x: Val::Percent(90.0),
        width: Val::Percent(10.0),
        height: Val::Percent(80.0),
        anchor: Anchor::TopRight,
        style: ItemStyle {
            corner_radius: Val::Vh(4.0),
            background_gradient: (RGB_PALETTE[0][0] * 0.1, RGB_PALETTE[0][0] * 0.6),
            ..default()
        },
        ..default()
    });
    {
        let _guard = pico.vstack(Val::Percent(0.0), Val::Percent(-1.0), true, &bottom_up_box);
        for row in 0..7 {
            pico.add(PicoItem {
                width: Val::Percent(100.0),
                height: Val::Percent(9.0),
                anchor: Anchor::BottomCenter,
                anchor_parent: Anchor::BottomCenter,
                style: ItemStyle {
                    corner_radius: Val::Vh(4.0),
                    background_color: RGB_PALETTE[0][row],
                    ..default()
                },
                parent: Some(bottom_up_box),
                ..default()
            });
        }
    }
}
