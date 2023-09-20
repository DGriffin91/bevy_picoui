use bevy::{prelude::*, sprite::Anchor};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
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

fn update(mut pico: ResMut<Pico>, windows: Query<&Window>) {
    let main_box = pico.add(PicoItem {
        x: Val::Percent(50.0),
        y: Val::Percent(50.0),
        width: Val::Vh(70.0),
        height: Val::Vh(30.0),
        anchor: Anchor::Center,
        style: ItemStyle {
            background_color: Color::rgb(0.1, 0.1, 0.1),
            ..default()
        },
        ..default()
    });
    let Ok(window) = windows.get_single() else {
        return;
    };

    {
        let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), &main_box);

        for i in 0..3 {
            let lane = pico.add(PicoItem {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0 / 3.0),
                anchor: Anchor::TopLeft,
                parent: Some(main_box),
                ..default()
            });
            {
                let _guard = pico.hstack(Val::Px(0.0), Val::Px(0.0), &lane);
                for j in 0..7 {
                    let cell = pico.add(PicoItem {
                        width: Val::Percent(100.0 / 7.0),
                        height: Val::Percent(100.0),
                        anchor: Anchor::TopLeft,
                        parent: Some(lane),
                        ..default()
                    });
                    {
                        let _guard = pico.stack_bypass();
                        let color = RGB_PALETTE[i][j];
                        let btn = pico.add(PicoItem {
                            x: Val::Percent(10.0),
                            y: Val::Percent(10.0),
                            width: Val::Percent(80.0),
                            height: Val::Percent(80.0),
                            style: ItemStyle {
                                corner_radius: Val::Percent(10.0),
                                background_color: color,
                                ..default()
                            },
                            anchor: Anchor::TopLeft,
                            anchor_parent: Anchor::TopLeft,
                            parent: Some(cell),
                            ..default()
                        });
                        if pico.hovered(&btn) {
                            if let Some(cursor_position) = window.cursor_position() {
                                let tooltip = pico.add(PicoItem {
                                    x: Val::Px(cursor_position.x),
                                    y: Val::Px(cursor_position.y + 20.0),
                                    width: Val::Vh(20.0),
                                    height: Val::Vh(15.0),
                                    style: ItemStyle {
                                        background_color: color,
                                        border_color: Color::WHITE,
                                        border_width: Val::Px(1.0),
                                        ..default()
                                    },
                                    anchor: Anchor::TopLeft,
                                    depth: Some(0.99),
                                    ..default()
                                });
                                let mut text = PicoItem {
                                    x: Val::Px(1.0),
                                    y: Val::Px(1.0),
                                    text: format!("{:#?}", color),
                                    anchor_parent: Anchor::Center,
                                    style: ItemStyle {
                                        text_alignment: TextAlignment::Left,
                                        text_color: Color::BLACK,
                                        ..default()
                                    },
                                    parent: Some(tooltip),
                                    ..default()
                                };
                                pico.add(text.clone());
                                text.style.text_color = Color::WHITE;
                                text.x = Val::Px(0.0);
                                text.y = Val::Px(0.0);
                                pico.add(text.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}
