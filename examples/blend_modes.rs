use bevy::{
    prelude::*,
    render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState},
    sprite::Anchor,
};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    widgets::basic_drag_widget,
    PicoPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PicoPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), Pico2dCamera));
}

fn update(
    mut pico: ResMut<Pico>,
    asset_server: ResMut<AssetServer>,
    mut image: Local<Option<Handle<Image>>>,
    mut red: Local<f32>,
    mut green: Local<f32>,
    mut blue: Local<f32>,
    mut char_input_events: EventReader<ReceivedCharacter>,
) {
    if image.is_none() {
        *image = Some(
            asset_server
                .load("images/generic-rpg-ui-inventario.png")
                .into(),
        );
    }
    let image = image.as_mut().unwrap();

    let controls = pico.add(PicoItem {
        depth: Some(0.99),
        x: Val::Px(30.0),
        y: Val::Px(30.0),
        width: Val::VMin(20.0),
        height: Val::VMin(10.0),
        anchor: Anchor::TopLeft,
        anchor_parent: Anchor::TopLeft,
        ..default()
    });

    let count = 3;

    let mut cdrag = |pico: &mut Pico, bg: Color, label: &str, value: f32, relative: bool| -> f32 {
        let parent = pico.add(PicoItem {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0 / count as f32),
            anchor: Anchor::TopLeft,
            parent: Some(controls),
            ..default()
        });
        basic_drag_widget(
            pico,
            parent,
            label,
            value,
            3.0,
            bg,
            &mut char_input_events,
            relative,
        )
    };

    {
        let _guard = pico.vstack(Val::Vh(1.0), Val::Vh(0.5), false, &controls);
        *red = cdrag(&mut pico, RED, "Red", *red, false).clamp(0.0, 1.0);
        *green = cdrag(&mut pico, GREEN, "Green", *green, false).clamp(0.0, 1.0);
        *blue = cdrag(&mut pico, BLUE, "Blue", *blue, false).clamp(0.0, 1.0);
    }

    let bg = pico.add(PicoItem {
        x: Val::Px(0.0),
        y: Val::Px(0.0),
        style: ItemStyle {
            corner_radius: Val::Percent(10.0),
            background_color: RGB_PALETTE[0][0] * 0.2,
            border_width: Val::Px(1.0),
            border_color: RGB_PALETTE[0][3],
            ..default()
        },
        width: Val::VMin(70.0),
        height: Val::VMin(70.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        ..default()
    });

    pico.add(PicoItem {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        style: ItemStyle {
            // For image to be fully opaque with the correct colors, the background needs to be white.
            background_color: Color::WHITE,
            image: Some(image.clone()),
            ..default()
        },
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        parent: Some(bg),
        ..default()
    });

    // Multiplicative blending
    pico.add(PicoItem {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        style: ItemStyle {
            background_color: Color::rgb(*red, *green, *blue),
            blend_state: Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::Zero,
                    dst_factor: BlendFactor::Src,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    src_factor: BlendFactor::Zero,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
            }),
            ..default()
        },
        parent: Some(bg),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        ..default()
    });
}

pub const RED: Color = Color::Rgba {
    red: 0.25,
    green: 0.18,
    blue: 0.18,
    alpha: 1.0,
};
pub const GREEN: Color = Color::Rgba {
    red: 0.18,
    green: 0.25,
    blue: 0.18,
    alpha: 1.0,
};
pub const BLUE: Color = Color::Rgba {
    red: 0.18,
    green: 0.18,
    blue: 0.25,
    alpha: 1.0,
};
