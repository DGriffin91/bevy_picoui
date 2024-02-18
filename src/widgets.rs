use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec2,
    prelude::*,
    sprite::Anchor,
};

use crate::{
    pico::{ItemIndex, ItemStyle, PicoItem},
    Pico,
};

// -------------------------
// Button example widget
// -------------------------

pub fn button(pico: &mut Pico, item: PicoItem) -> ItemIndex {
    let index = pico.add(item);
    let c = pico.get(&index).style.background_color;
    pico.get_mut(&index).style.background_color = if pico.hovered(&index) {
        c + Color::rgba(0.06, 0.06, 0.06, 0.0)
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
) -> ItemIndex {
    let index = pico.add(item);
    let mut c = pico.get(&index).style.background_color;
    if pico.clicked(&index) {
        *toggle_state = !*toggle_state;
    }
    if *toggle_state {
        c = enabled_bg;
    }
    pico.get_mut(&index).style.background_color = if pico.hovered(&index) {
        c + Color::rgb(0.08, 0.08, 0.08)
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
pub fn hr(pico: &mut Pico, width: Val, height: Val, parent: Option<ItemIndex>) -> ItemIndex {
    pico.add(PicoItem {
        uv_position: vec2(0.5, 0.0),
        width,
        height,
        style: ItemStyle {
            background_color: Color::rgba(1.0, 1.0, 1.0, 0.04),
            ..default()
        },
        anchor: Anchor::TopCenter,
        parent,
        ..default()
    })
}

// -------------------------
// Value drag example widget
// -------------------------

#[allow(clippy::too_many_arguments)]
pub fn drag_value(
    pico: &mut Pico,
    scale: f32,
    value: f32,
    drag_index: ItemIndex,
    char_input_events: Option<&mut EventReader<ReceivedCharacter>>,
) -> f32 {
    let mut value = value;
    let mut drag_bg = pico.get_mut(&drag_index).style.background_color;

    let mut dragging = false;
    if let Some(state) = pico.get_state(&drag_index) {
        if let Some(drag) = state.drag {
            let delta = drag.delta();
            value += (delta.x - delta.y) * scale;
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
        let released = pico.released(&drag_index);
        let mut reset = false;
        let mut apply = false;
        let mut selected = false;
        if let Some(state) = pico.get_state_mut(&drag_index) {
            let mut just_selected = false;
            if state.storage.is_none() {
                state.storage = Some(Box::<String>::default());
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
                        for e in char_input_events.read() {
                            let char = e.char.chars().next().unwrap();
                            if char == esc {
                                reset = true;
                            } else if char == backspace {
                                s.pop();
                            } else if char == enter {
                                apply = true;
                                break;
                            } else if char.is_ascii_digit() || char == '.' || char == '-' {
                                s.push(char);
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
            pico.get_mut(&drag_index).text = current_string + "|";
        }
        if selected {
            drag_bg = drag_bg + Color::rgba(0.25, 0.25, 0.25, 0.0);
        }
    }
    pico.get_mut(&drag_index).style.background_color = if pico.hovered(&drag_index) || dragging {
        drag_bg + Color::rgba(0.06, 0.06, 0.06, 0.0)
    } else {
        drag_bg
    };
    value
}

// ---------------------------------------------------------
// Basic example drag widget with label in horizontal layout
// ---------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn basic_drag_widget(
    pico: &mut Pico,
    parent: ItemIndex,
    label: &str,
    value: f32,
    scale: f32,
    bg: Color,
    char_input_events: &mut EventReader<ReceivedCharacter>,
    relative: bool,
) -> f32 {
    let _guard = pico.hstack(Val::Percent(5.0), Val::Percent(1.0), false, &parent);
    // Label Text
    pico.add(PicoItem {
        text: label.to_string(),
        width: Val::Percent(60.0),
        height: Val::Percent(100.0),
        style: ItemStyle {
            anchor_text: Anchor::CenterLeft,
            ..default()
        },
        anchor: Anchor::TopLeft,
        parent: Some(parent),
        ..default()
    });
    // Drag box
    let drag_index = pico.add(PicoItem {
        text: format!("{:.2}", value),
        width: Val::Percent(30.0),
        height: Val::Percent(100.0),
        style: ItemStyle {
            corner_radius: Val::Percent(10.0),
            background_color: bg,
            ..default()
        },
        anchor: Anchor::TopLeft,
        parent: Some(parent),
        ..default()
    });
    let value = drag_value(pico, scale, value, drag_index, Some(char_input_events));
    if relative {
        // Show relative value while dragging drag
        if let Some(state) = pico.get_state_mut(&drag_index) {
            if let Some(drag) = state.drag {
                pico.get_mut(&drag_index).text = format!("{:.2}", drag.total_delta().x * scale)
            }
        }
    }
    value
}

// --------------------------
// Example scroll area widget
// --------------------------

// TODO don't use percent for button heights, content area etc... either make configurable or Vh

pub struct ScrollAreaWidget {
    pub items: Vec<ItemIndex>,
    pub scroll_widget: ItemIndex,
    pub content_area: ItemIndex,
    pub scroll_bar_area: ItemIndex,
    pub up_btn: ItemIndex,
    pub down_btn: ItemIndex,
    pub lane: ItemIndex,
    pub handle: ItemIndex,
    pub position: i32,
    pub fscroll_position: f32,
    pub scroll_updated: bool,
}

impl ScrollAreaWidget {
    pub fn new(
        pico: &mut Pico,
        scroll_range: i32,
        max_items_to_show: i32,
        id: u64,
        parent: ItemIndex,
        initial_scroll_position: Option<i32>,
        mouse_wheel_events: &mut EventReader<MouseWheel>,
    ) -> ScrollAreaWidget {
        let mut items = Vec::new();
        let content_area;
        let scroll_bar_area;
        let up_btn;
        let down_btn;
        let lane;
        let handle;
        let mut scroll_position = 0;
        let mut fscroll_position = 0.0;
        let mut scroll_updated = false;
        let mut fscroll_updated = false;

        let scroll_widget = pico.add(PicoItem {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            anchor: Anchor::TopLeft,
            anchor_parent: Anchor::TopLeft,
            parent: Some(parent),
            ..default()
        });

        if let Some(state) = pico.get_state_mut(&scroll_widget) {
            if state.storage.is_none() {
                if let Some(initial_scroll_position) = initial_scroll_position {
                    scroll_position = initial_scroll_position;
                    fscroll_position = scroll_position as f32 / scroll_range as f32;
                    state.storage = Some(Box::new((fscroll_position, scroll_position)));
                }
            }
        }

        {
            let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), false, &scroll_widget);

            if let Some(state) = pico.get_state_mut(&scroll_widget) {
                if let Some(storage) = &mut state.storage {
                    let scroll_data = storage.downcast_mut::<(i32, f32)>().unwrap();
                    scroll_position = scroll_data.0;
                    fscroll_position = scroll_data.1;
                }
            }

            {
                let _guard = pico.hstack(Val::Px(0.0), Val::Px(0.0), true, &scroll_widget);
                scroll_bar_area = pico.add(PicoItem {
                    width: Val::Vh(2.5),
                    height: Val::Percent(100.0),
                    anchor_parent: Anchor::TopRight,
                    anchor: Anchor::TopRight,
                    parent: Some(scroll_widget),
                    ..default()
                });
                content_area = pico.add(PicoItem {
                    uv_size: vec2(pico.remaining_stack_space(), 1.0),
                    anchor_parent: Anchor::TopRight,
                    anchor: Anchor::TopRight,
                    parent: Some(scroll_widget),
                    ..default()
                });
                {
                    let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), false, &scroll_bar_area);
                    up_btn = pico.add(PicoItem {
                        width: Val::Percent(100.0),
                        height: Val::Percent(5.0),
                        style: ItemStyle {
                            background_color: Color::rgb(0.2, 0.2, 0.2),
                            ..default()
                        },
                        anchor_parent: Anchor::TopLeft,
                        anchor: Anchor::TopLeft,
                        parent: Some(scroll_bar_area),
                        ..default()
                    });
                    lane = pico.add(PicoItem {
                        width: Val::Percent(100.0),
                        height: Val::Percent(90.0),
                        anchor_parent: Anchor::TopLeft,
                        anchor: Anchor::TopLeft,
                        parent: Some(scroll_bar_area),
                        ..default()
                    });
                    down_btn = pico.add(PicoItem {
                        width: Val::Percent(100.0),
                        height: Val::Percent(5.0),
                        style: ItemStyle {
                            background_color: Color::rgb(0.2, 0.2, 0.2),
                            ..default()
                        },
                        anchor_parent: Anchor::TopLeft,
                        anchor: Anchor::TopLeft,
                        parent: Some(scroll_bar_area),
                        ..default()
                    });

                    {
                        let handle_height_vh = 4.0;
                        let lane_bbox = pico.get(&lane).get_bbox();
                        let lane_height = (lane_bbox.w - lane_bbox.y) - handle_height_vh / 100.0;
                        if let Some(state) = pico.state.get(&id) {
                            if let Some(drag) = state.drag {
                                let delta = drag.delta();
                                fscroll_position =
                                    (fscroll_position + delta.y / lane_height).clamp(0.0, 1.0);
                                scroll_position = (fscroll_position * scroll_range as f32) as i32;
                                fscroll_updated = true;
                            };
                        }
                        if pico.hovered(&scroll_widget) {
                            for event in mouse_wheel_events.read() {
                                scroll_position = (scroll_position
                                    + match event.unit {
                                        MouseScrollUnit::Line => -event.y,
                                        MouseScrollUnit::Pixel => -event.y / 10.0, //TODO: idk about scale
                                    } as i32)
                                    .clamp(0, scroll_range);
                                scroll_updated = true;
                            }
                        }
                        let handle_abs_pos = (fscroll_position * lane_height) * 100.0;
                        let _guard = pico.stack_bypass();
                        handle = pico.add(PicoItem {
                            y: Val::Vh(handle_abs_pos + handle_height_vh * 0.5),
                            width: Val::Percent(100.0),
                            height: Val::Vh(handle_height_vh),
                            parent: Some(lane),
                            anchor: Anchor::Center,
                            anchor_parent: Anchor::TopCenter,
                            spatial_id: Some(id), // Manually set id
                            ..default()
                        });
                    }
                    if pico.clicked(&up_btn) {
                        scroll_position = (scroll_position - 1).max(0);
                        scroll_updated = true;
                    }
                    if pico.clicked(&down_btn) {
                        scroll_position = (scroll_position + 1).min(scroll_range);
                        scroll_updated = true;
                    }
                }
                if scroll_updated {
                    fscroll_position = scroll_position as f32 / scroll_range as f32;
                }
                {
                    let _guard = pico.vstack(Val::Px(0.0), Val::Px(0.0), false, &content_area);
                    let scroll_position = scroll_position as usize;
                    for _ in scroll_position..scroll_position + max_items_to_show as usize {
                        items.push(pico.add(PicoItem {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0 / max_items_to_show as f32),
                            anchor: Anchor::TopLeft,
                            anchor_parent: Anchor::TopLeft,
                            parent: Some(content_area),
                            ..default()
                        }));
                    }
                }
            }
            if scroll_updated || fscroll_updated {
                if let Some(state) = pico.get_state_mut(&scroll_widget) {
                    state.storage = Some(Box::new((scroll_position, fscroll_position)));
                }
            }
        }
        ScrollAreaWidget {
            items,
            scroll_widget,
            content_area,
            scroll_bar_area,
            up_btn,
            down_btn,
            lane,
            handle,
            position: scroll_position,
            fscroll_position,
            scroll_updated: scroll_updated || fscroll_updated,
        }
    }
}
