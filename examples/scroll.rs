use bevy::{input::mouse::MouseWheel, prelude::*, sprite::Anchor};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{ItemStyle, Pico, Pico2dCamera, PicoItem},
    widgets::ScrollAreaWidget,
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

fn update(mut pico: ResMut<Pico>, mut mouse_wheel_events: EventReader<MouseWheel>) {
    let indices: Vec<_> = (0..3).flat_map(|i| (0..7).map(move |j| (i, j))).collect();
    let total_items = 3 * 7;
    let max_items_to_show = 10;
    let scroll_range = total_items - max_items_to_show;

    let scroll_container = pico.add(PicoItem {
        width: Val::Vh(50.0),
        height: Val::Vh(50.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        style: ItemStyle {
            background_color: Color::rgb(0.1, 0.1, 0.1),
            ..default()
        },
        ..default()
    });

    let scroll = ScrollAreaWidget::new(
        &mut pico,
        scroll_range,
        max_items_to_show,
        // Need to use a consistent id for keeping scroll state
        098743542350897,
        scroll_container,
        None,
        &mut mouse_wheel_events,
    );

    pico.get_mut(&scroll.handle).style = ItemStyle {
        corner_radius: Val::Percent(25.0),
        background_color: Color::rgb(0.4, 0.4, 0.4),
        ..default()
    };

    let circle = PicoItem {
        width: Val::Percent(25.0),
        height: Val::Percent(25.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        style: ItemStyle {
            corner_radius: Val::Percent(50.0),
            background_color: Color::rgba(1.0, 1.0, 1.0, 0.1),
            ..default()
        },
        parent: Some(scroll.up_btn),
        ..default()
    };

    let mut circle2 = circle.clone();
    circle2.parent = Some(scroll.down_btn);

    pico.add(circle);
    pico.add(circle2);

    for (i, index) in scroll.items.iter().enumerate() {
        let (i, j) = indices[i + scroll.position as usize];
        let item = pico.get_mut(index);
        let color = RGB_PALETTE[i][j];
        item.style.corner_radius = Val::Percent(30.0);
        item.style.background_color = color;
    }
}
