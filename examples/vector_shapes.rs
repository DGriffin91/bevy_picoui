use std::{f32::consts::TAU, time::Duration};

use bevy::{asset::ChangeWatcher, math::*, prelude::*, render::view::RenderLayers, sprite::Anchor};

use bevy_picoui::{
    pico::{Pico, Pico2dCamera, PicoItem},
    widgets::drag_value,
    PicoPlugin,
};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::*, Shape2dPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
            ..default()
        }))
        .insert_resource(GizmoConfig {
            render_layers: RenderLayers::layer(1),
            ..default()
        })
        .add_plugins(Shape2dPlugin::default())
        .add_plugins((PicoPlugin {
            create_default_2d_cam_with_order: Some(1),
        },))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 5.5, 10.0)
                .looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
            ..default()
        },
        Pico2dCamera,
        RenderLayers::all(),
    ));
}

fn update(
    mut pico: ResMut<Pico>,
    mut painter: ShapePainter,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut values: Local<Option<[f32; 9]>>,
) {
    if values.is_none() {
        let mut v = [0.0; 9];
        v[1] = 1.0;
        v[5] = 0.6;
        v[8] = 0.9;
        *values = Some(v);
    }
    let values = values.as_mut().unwrap();

    let main_box = pico.add(PicoItem {
        depth: Some(0.01),
        x: Val::Percent(0.0),
        y: Val::Percent(0.0),
        width: Val::VMin(50.0),
        height: Val::VMin(50.0),
        corner_radius: Val::Px(10.0),
        border_width: Val::Px(1.0),
        border_color: Color::WHITE,
        anchor: Anchor::Center,
        anchor_parent: Anchor::Center,
        background: SLATE,
        ..default()
    });
    let mut points = Vec::new();
    for (i, parent_anchor) in [
        Anchor::Center,
        Anchor::BottomLeft,
        Anchor::BottomCenter,
        Anchor::BottomRight,
        Anchor::CenterLeft,
        Anchor::CenterRight,
        Anchor::TopLeft,
        Anchor::TopCenter,
        Anchor::TopRight,
    ]
    .iter()
    .enumerate()
    {
        // 0.0 for center anchors, multiplied by x,y so it is not offset for center axis
        let center_anchor = (parent_anchor.as_vec() * 2.0).abs();
        let drag_index = pico.add(PicoItem {
            text: format!("{:.2}", values[i]),
            x: Val::Vh(2.0 * center_anchor.x),
            y: Val::Vh(2.0 * center_anchor.y),
            width: Val::Vh(5.0),
            height: Val::Vh(5.0),
            anchor: parent_anchor.clone(),
            anchor_parent: parent_anchor.clone(),
            parent: Some(main_box),
            ..default()
        });

        values[i] = drag_value(
            &mut pico,
            1.5,
            values[i],
            drag_index,
            Some(&mut char_input_events),
        )
        .clamp(0.0, 1.0);

        let p = pico.center(drag_index);
        points.push(p);
        let ws_p = pico.uv_position_to_ws_px(p);
        painter.color = Color::WHITE;
        painter.set_translation(ws_p.extend(pico.auto_depth()));
        painter.hollow = true;
        painter.cap = Cap::Round;
        painter.thickness = pico.val_y_px(Val::Vh(0.1)).max(1.0);
        let start = -TAU * 0.33;
        let end = TAU * 0.66;
        painter.arc(pico.val_y_px(Val::Vh(3.0)), start, start + end * values[i]);
    }

    painter.set_translation(Vec2::ZERO.extend(pico.auto_depth()));
    for i in 0..points.len() {
        let a = pico.uv_position_to_ws_px(points[i]);
        painter.color = Color::rgba(
            1.0,
            points[i].y * 1.2,
            i as f32 / 10.0,
            (values[i] * 0.5).powf(2.0),
        );
        for j in 0..points.len() {
            let b = pico.uv_position_to_ws_px(points[j]);
            if i != j {
                painter.line(a.extend(0.0), b.extend(0.0));
            }
        }
    }
}

// ------
// Colors
// ------

pub const SLATE: Color = Color::Rgba {
    red: 0.156,
    green: 0.239,
    blue: 0.231,
    alpha: 1.0,
};

pub const CURRENT: Color = Color::Rgba {
    red: 0.098,
    green: 0.447,
    blue: 0.470,
    alpha: 1.0,
};

pub const BURNT_RED: Color = Color::Rgba {
    red: 0.466,
    green: 0.180,
    blue: 0.145,
    alpha: 1.0,
};

pub const OILVINE: Color = Color::Rgba {
    red: 0.549,
    green: 0.702,
    blue: 0.412,
    alpha: 1.0,
};
