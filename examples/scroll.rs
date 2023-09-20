use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    sprite::Anchor,
};

use bevy_picoui::{
    palette::RGB_PALETTE,
    pico::{lerp, ItemStyle, Pico, Pico2dCamera, PicoItem},
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

fn update(
    mut pico: ResMut<Pico>,
    mut scroll_position: Local<i32>,
    mut fscroll_position: Local<f32>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let scroll_widget = pico.add(PicoItem {
        width: Val::Vh(50.0),
        height: Val::Vh(50.0),
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        style: ItemStyle {
            background: Color::rgb(0.1, 0.1, 0.1),
            ..default()
        },
        ..default()
    });

    {
        let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), scroll_widget);

        let indices: Vec<_> = (0..3).flat_map(|i| (0..7).map(move |j| (i, j))).collect();
        let total_items = 3 * 7;
        let max_items_to_show = 10;
        let scroll_range = total_items - max_items_to_show;

        {
            let _guard = pico.hstack(Val::Px(0.0), Val::Px(0.0), scroll_widget);
            let content_area = pico.add(PicoItem {
                width: Val::Percent(95.0),
                height: Val::Percent(100.0),
                anchor_parent: Anchor::TopLeft,
                anchor: Anchor::TopLeft,
                parent: Some(scroll_widget),
                ..default()
            });
            let scroll_bar_area = pico.add(PicoItem {
                width: Val::Percent(5.0),
                height: Val::Percent(100.0),
                anchor_parent: Anchor::TopLeft,
                anchor: Anchor::TopLeft,
                parent: Some(scroll_widget),
                ..default()
            });
            {
                let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), scroll_bar_area);
                let up_btn = pico.add(PicoItem {
                    text: String::from("^"),
                    width: Val::Percent(100.0),
                    height: Val::Percent(5.0),
                    style: ItemStyle {
                        background: Color::rgb(0.2, 0.2, 0.2),
                        ..default()
                    },
                    anchor_parent: Anchor::TopLeft,
                    anchor: Anchor::TopLeft,
                    parent: Some(scroll_bar_area),
                    ..default()
                });
                let lane = pico.add(PicoItem {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    anchor_parent: Anchor::TopLeft,
                    anchor: Anchor::TopLeft,
                    parent: Some(scroll_bar_area),
                    ..default()
                });

                {
                    let handle_height_vh = 4.0;
                    let lane_bbox = pico.get(lane).get_bbox();
                    let lane_height = (lane_bbox.w - lane_bbox.y) - handle_height_vh / 100.0;
                    // Need to use a consistent id, usually the id is generated from spatial components of the item
                    let id = 098743542350897;
                    if let Some(state) = pico.state.get(&id) {
                        if let Some(drag) = state.drag {
                            let delta = drag.delta();
                            *fscroll_position =
                                (*fscroll_position + delta.y / lane_height).clamp(0.0, 1.0);
                            *scroll_position = (*fscroll_position * scroll_range as f32) as i32;
                        };
                    }
                    if pico.hovered(scroll_widget) {
                        for event in mouse_wheel_events.iter() {
                            *scroll_position = (*scroll_position
                                + match event.unit {
                                    MouseScrollUnit::Line => -event.y,
                                    MouseScrollUnit::Pixel => -event.y / 10.0, //TODO: idk about scale
                                } as i32)
                                .clamp(0, scroll_range);
                            *fscroll_position = *scroll_position as f32 / scroll_range as f32;
                        }
                    }
                    let handle_abs_pos = (*fscroll_position * lane_height) * 100.0;
                    let _guard = pico.stack_bypass();
                    pico.add(PicoItem {
                        y: Val::Vh(handle_abs_pos + handle_height_vh * 0.5),
                        width: Val::Percent(100.0),
                        height: Val::Vh(handle_height_vh),
                        style: ItemStyle {
                            corner_radius: Val::Percent(25.0),
                            background: Color::rgb(0.4, 0.4, 0.4),
                            ..default()
                        },
                        parent: Some(lane),
                        anchor: Anchor::Center,
                        anchor_parent: Anchor::TopCenter,
                        spatial_id: Some(id), // Manually set id
                        ..default()
                    });
                }
                let down_btn = pico.add(PicoItem {
                    text: String::from("v"),
                    width: Val::Percent(100.0),
                    height: Val::Percent(5.0),
                    style: ItemStyle {
                        background: Color::rgb(0.2, 0.2, 0.2),
                        ..default()
                    },
                    anchor_parent: Anchor::TopLeft,
                    anchor: Anchor::TopLeft,
                    parent: Some(scroll_bar_area),
                    ..default()
                });
                if pico.clicked(up_btn) {
                    *scroll_position = (*scroll_position - 1).max(0);
                    *fscroll_position = *scroll_position as f32 / scroll_range as f32;
                }
                if pico.clicked(down_btn) {
                    *scroll_position = (*scroll_position + 1).min(scroll_range as i32);
                    *fscroll_position = *scroll_position as f32 / scroll_range as f32;
                }
            }
            {
                let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), content_area);
                let scroll_position = *scroll_position as usize;
                for (i, j) in
                    &indices[scroll_position..scroll_position + max_items_to_show as usize]
                {
                    let color = RGB_PALETTE[*i][*j];
                    pico.add(PicoItem {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0 / max_items_to_show as f32),
                        style: ItemStyle {
                            corner_radius: Val::Percent(10.0),
                            background: color,
                            ..default()
                        },
                        anchor: Anchor::TopLeft,
                        anchor_parent: Anchor::TopLeft,
                        parent: Some(content_area),
                        ..default()
                    });
                }
            }
        }
    }
}
