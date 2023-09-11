use bevy::{math::Vec3Swizzles, prelude::*, sprite::Anchor, text::BreakLineOn};
use std::hash::Hasher;

use bevy::utils::HashMap;

use crate::{prepare_view, View};

pub struct ImTextPlugin;

impl Plugin for ImTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, render_imtext.after(prepare_view));
    }
}

// Only supports one camera.
#[derive(Component)]
pub struct Im3dTextCamera;

#[derive(Component, Clone)]
pub struct Im3dText {
    pub position: Vec3,
    pub text: String,
    pub font_size: f32,
    pub color: Color,
    pub alignment: TextAlignment,
    pub anchor: Anchor,
}

impl Im3dText {
    pub fn new(position: Vec3, text: &str) -> Im3dText {
        Im3dText {
            position,
            text: text.to_string(),
            font_size: 14.0,
            color: Color::WHITE,
            alignment: TextAlignment::Center,
            anchor: Anchor::Center,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
}

impl std::hash::Hash for Im3dText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);
    }
}

impl PartialEq for Im3dText {
    fn eq(&self, other: &Im3dText) -> bool {
        self.text == other.text
            && self.position == other.position
            && self.alignment == other.alignment
            && self.anchor.as_vec() == other.anchor.as_vec()
    }
}

impl Eq for Im3dText {}

fn render_imtext(
    //render_layers: Res<ImTextRenderLayers>,
    mut commands: Commands,
    imtexts: Query<(Entity, &Im3dText)>,
    mut cache: Local<HashMap<Im3dText, (Entity, bool)>>,
    camera: Query<&View, With<Im3dTextCamera>>,
    mut text_entites: Query<&mut Transform, With<Text>>,
    windows: Query<&Window>,
) {
    let Ok(view) = camera.get_single() else {
        return;
    };
    let Ok(window) = windows.get_single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    // Mark cached items as not in use
    for (_, (_, in_use)) in cache.iter_mut() {
        *in_use = false;
    }

    for (im_entity, imtext) in &imtexts {
        let text_ndc = view.position_world_to_ndc(imtext.position);
        let text_pos = (text_ndc.xy() * window_size * 0.5).extend(1.0);
        if let Some((entity, in_use)) = cache.get_mut(imtext) {
            // If a Im3dText in the cache matches one created this frame, mark as in use
            *in_use = true;
            let Ok(mut trans) = text_entites.get_mut(*entity) else {
                continue;
            };
            trans.translation = text_pos;
        } else {
            let entity = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection::new(
                            imtext.text.clone(),
                            TextStyle {
                                font_size: imtext.font_size,
                                color: imtext.color,
                                ..default()
                            },
                        )],
                        alignment: TextAlignment::Center,
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    text_anchor: imtext.anchor.clone(),
                    transform: Transform::from_translation(text_pos),
                    ..default()
                })
                .id();

            cache.insert(imtext.clone(), (entity, true));
        }
        commands.entity(im_entity).despawn();
    }

    for (_, (entity, in_use)) in cache.iter_mut() {
        // Remove cached Im3dTexts that are no longer in use
        if !*in_use {
            commands.entity(*entity).despawn_recursive();
        }
    }
}
