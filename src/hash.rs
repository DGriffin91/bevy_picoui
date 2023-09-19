use std::hash::{Hash, Hasher};

use bevy::{prelude::*, sprite::Anchor};

pub fn hash_color<H: Hasher>(color: &Color, state: &mut H) {
    color.r().to_bits().hash(state);
    color.g().to_bits().hash(state);
    color.b().to_bits().hash(state);
    color.a().to_bits().hash(state);
}

pub fn hash_vec2<H: Hasher>(v: &Vec2, state: &mut H) {
    v.x.to_bits().hash(state);
    v.y.to_bits().hash(state);
}

pub fn hash_vec3<H: Hasher>(v: &Vec3, state: &mut H) {
    v.x.to_bits().hash(state);
    v.y.to_bits().hash(state);
    v.z.to_bits().hash(state);
}

pub fn hash_vec4<H: Hasher>(v: &Vec4, state: &mut H) {
    v.x.to_bits().hash(state);
    v.y.to_bits().hash(state);
    v.z.to_bits().hash(state);
    v.w.to_bits().hash(state);
}

pub fn hash_val<H: Hasher>(v: &Val, state: &mut H) {
    match v {
        Val::Auto => &0.0,
        Val::Px(f) => f,
        Val::Percent(f) => f,
        Val::Vw(f) => f,
        Val::Vh(f) => f,
        Val::VMin(f) => f,
        Val::VMax(f) => f,
    }
    .to_bits()
    .hash(state);
    match v {
        Val::Auto => 0,
        Val::Px(_) => 1,
        Val::Percent(_) => 2,
        Val::Vw(_) => 3,
        Val::Vh(_) => 4,
        Val::VMin(_) => 5,
        Val::VMax(_) => 6,
    }
    .hash(state);
}

pub fn hash_anchor<H: Hasher>(anchor: &Anchor, state: &mut H) {
    match anchor {
        Anchor::Center => 0.hash(state),
        Anchor::BottomLeft => 1.hash(state),
        Anchor::BottomCenter => 2.hash(state),
        Anchor::BottomRight => 3.hash(state),
        Anchor::CenterLeft => 4.hash(state),
        Anchor::CenterRight => 5.hash(state),
        Anchor::TopLeft => 6.hash(state),
        Anchor::TopCenter => 7.hash(state),
        Anchor::TopRight => 8.hash(state),
        Anchor::Custom(point) => {
            point.x.to_bits().hash(state);
            point.y.to_bits().hash(state);
        }
    }
}
