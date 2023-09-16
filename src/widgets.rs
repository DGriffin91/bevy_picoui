use bevy::{math::vec2, prelude::*, sprite::Anchor};

use crate::{Pico, PicoItem};

// -------------------------
// Button example widget
// -------------------------

pub fn button(pico: &mut Pico, item: PicoItem) -> usize {
    let index = pico.items.len();
    let pico = pico.add(item);
    let c = pico.get(index).background;
    pico.get_mut(index).background = if pico.hovered(index) {
        c + Vec4::splat(0.06)
    } else {
        c
    };
    index
}

// -------------------------
// Toggle Button example widget
// -------------------------

pub fn toggle_button(
    pico: &mut Pico,
    item: PicoItem,
    enabled_bg: Color,
    toggle_state: &mut bool,
) -> usize {
    let index = pico.items.len();
    let pico = pico.add(item);
    let mut c = pico.get(index).background;
    if pico.clicked(index) {
        *toggle_state = !*toggle_state;
    }
    if *toggle_state {
        c = enabled_bg;
    }
    pico.get_mut(index).background = if pico.hovered(index) {
        c + Vec4::splat(0.06)
    } else {
        c
    };
    index
}

// -------------------------
// Horizontal ruler example widget
// -------------------------

/// Width is relative to parent. Height-y parent is removed so height is only relative to screen height
/// so HRs are of a consistent height for the same input height.
pub fn hr(pico: &mut Pico, width: Val, height: Val, parent: Option<usize>) -> usize {
    let index = pico.last();
    pico.add(PicoItem {
        uv_position: vec2(0.5, 0.0),
        width,
        height,
        background: Color::rgba(1.0, 1.0, 1.0, 0.04),
        anchor: Anchor::TopCenter,
        parent,
        ..default()
    });
    index
}

// -------------------------
// Value drag example widget
// -------------------------

pub struct DragValue {
    pub value: f32,
    pub drag_index: usize,
}

pub fn drag_value(
    pico: &mut Pico,
    drag_bg: Color,
    drag_width: Val,
    corner_radius: Val,
    scale: f32,
    value: f32,
    parent: usize,
    char_input_events: Option<&mut EventReader<ReceivedCharacter>>,
) -> DragValue {
    let mut value = value;
    let mut drag_bg = drag_bg;

    let drag_index = pico.items.len();

    pico.add(PicoItem {
        // TODO user or auto precision
        text: format!("{:.2}", value),
        width: drag_width,
        height: Val::Percent(100.0),
        corner_radius,
        anchor: Anchor::TopLeft,
        parent: Some(parent),
        ..default()
    });

    let mut dragging = false;
    if let Some(state) = pico.get_state(drag_index) {
        if let Some(drag) = state.drag {
            value = drag.delta().x * scale + value;
            dragging = true;
        }
    };
    if let Some(char_input_events) = char_input_events {
        let mouse_just_pressed = if let Some(mouse_button_input) = &pico.mouse_button_input {
            mouse_button_input.any_just_pressed([
                MouseButton::Left,
                MouseButton::Right,
                MouseButton::Middle,
            ])
        } else {
            false
        };
        let mut current_string = None;
        let released = pico.released(drag_index);
        let mut reset = false;
        let mut apply = false;
        let mut selected = false;
        if let Some(state) = pico.get_state_mut(drag_index) {
            let mut just_selected = false;
            if state.storage.is_none() {
                state.storage = Some(Box::new(String::new()));
            }
            if !dragging && released {
                state.selected = true;
                just_selected = true;
            }
            if state.selected {
                selected = true;
                let backspace = char::from_u32(0x08).unwrap();
                let esc = char::from_u32(0x1b).unwrap();
                let enter = '\r';
                if let Some(storage) = &mut state.storage {
                    let s = storage.downcast_mut::<String>().unwrap();
                    if mouse_just_pressed && !just_selected {
                        apply = true;
                    } else {
                        // TODO: usually when a text field like this is first selected it would have all of the
                        // text in the field selected, so typing anything would overwrite the existing value
                        // or the cursor could be moved to preserve the value.
                        //if just_selected {
                        //    // TODO user or auto precision
                        //    *s = format!("{:.2}", value);
                        //}
                        for e in char_input_events.iter() {
                            if e.char == esc {
                                reset = true;
                            } else if e.char == backspace {
                                s.pop();
                            } else if e.char == enter {
                                apply = true;
                                break;
                            } else if e.char.is_digit(10) || e.char == '.' || e.char == '-' {
                                s.push(e.char);
                            }
                        }
                        current_string = Some(s.clone());
                    }
                    if apply {
                        if let Ok(parse_val) = s.parse::<f32>() {
                            value = parse_val;
                        }
                        reset = true;
                    }
                    if reset {
                        state.selected = false;
                        *s = String::new();
                    }
                }
            }
        }
        if let Some(current_string) = current_string {
            pico.get_mut(drag_index).text = current_string + "|";
        }
        if selected {
            drag_bg += Vec4::splat(0.25);
        }
    }
    pico.get_mut(drag_index).background = if pico.hovered(drag_index) || dragging {
        drag_bg + Vec4::splat(0.06)
    } else {
        drag_bg
    };
    DragValue { value, drag_index }
}
