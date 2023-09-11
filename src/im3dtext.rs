use bevy::{math::Vec3Swizzles, prelude::*};
use std::hash::Hasher;

use bevy::utils::HashMap;

use crate::{ndc_to_uv, prepare_view, View};

pub struct ImTextPlugin;

// RenderLayers are not supported for UI yet
//#[derive(Resource, Default)]
//pub struct ImTextRenderLayers(pub RenderLayers);

impl Plugin for ImTextPlugin {
    fn build(&self, app: &mut App) {
        app //.init_resource::<ImTextRenderLayers>()
            .add_systems(PostUpdate, render_imtext.after(prepare_view));
    }
}

// Only supports one camera.
#[derive(Component)]
pub struct Im3dTextCamera;

#[derive(Component, Clone, PartialEq)]
pub struct Im3dText {
    pub position: Vec3,
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

impl Im3dText {
    pub fn new(position: Vec3, text: &str) -> Im3dText {
        Im3dText {
            position,
            text: text.to_string(),
            font_size: 14.0,
            color: Color::WHITE,
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
}

impl std::hash::Hash for Im3dText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_size.to_bits().hash(state);
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);
    }
}
impl Eq for Im3dText {}

fn render_imtext(
    //render_layers: Res<ImTextRenderLayers>,
    mut commands: Commands,
    imtexts: Query<(Entity, &Im3dText)>,
    mut cache: Local<HashMap<Im3dText, (Entity, bool)>>,
    camera: Query<&View, With<Im3dTextCamera>>,
    mut text_entites: Query<(&mut Text, &mut Style)>,
) {
    let Ok(view) = camera.get_single() else {
        return;
    };

    // Mark cached items as not in use
    for (_, (_, in_use)) in cache.iter_mut() {
        *in_use = false;
    }

    for (im_entity, imtext) in &imtexts {
        let text_uv = ndc_to_uv(view.position_world_to_ndc(imtext.position).xy());
        if let Some((entity, in_use)) = cache.get_mut(imtext) {
            // If a Im3dText in the cache matches one created this frame, mark as in use
            *in_use = true;
            let Ok((mut text, mut style)) = text_entites.get_mut(*entity) else {
                continue;
            };
            *text = text.clone().with_alignment(TextAlignment::Center);
            style.top = Val::Percent(text_uv.y * 100.0);
            style.left = Val::Percent(text_uv.x * 100.0);
        } else {
            // Create a new entity if one wasn't found in the cache
            let entity = commands
                .spawn(
                    TextBundle::from_section(
                        imtext.text.clone(),
                        TextStyle {
                            font_size: imtext.font_size,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(text_uv.y * 100.0),
                        left: Val::Percent(text_uv.x * 100.0),
                        //top: Val::Percent(50.0),
                        //left: Val::Percent(50.0),
                        // TODO still not centered. When this is figured out add to Im3dText.
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        ..default()
                    })
                    .with_text_alignment(TextAlignment::Center),
                )
                //.insert(render_layers.0)
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
