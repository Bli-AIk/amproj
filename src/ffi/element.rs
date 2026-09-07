use std::ffi::{CString, c_char};
use std::ptr;

use glam::Vec4;

use crate::animation::{
    interpolate_color, interpolate_float, interpolate_vec2, interpolate_vec3, parse_keyframe_color,
};
use crate::coord::{CoordMappingConfig, apply_coord_mapping, multiply_4x4_column_major};
use crate::schema::{
    AmAnimatedColor, AmAnimatedFloat, AmAnimatedVec2, AmEffect, AmFillColor, AmGradient,
    AmKeyframe, AmProperty, AmTransform, parse_color, parse_vec2,
};

use super::types::{EffectInstance, FlatElement, MAX_EFFECTS_PER_ELEMENT};

#[derive(Debug, Clone, Copy)]
pub(super) struct TransformValues {
    position: [f32; 2],
    rotation_deg: f32,
    scale: [f32; 2],
    pivot: [f32; 2],
    opacity: f32,
}

pub(super) fn transform_values(
    transform: &AmTransform,
    t: f32,
    default_position: [f32; 2],
) -> TransformValues {
    let location = interpolate_vec3(&transform.location, t)
        .or(transform.location.value)
        .unwrap_or([default_position[0], default_position[1], 0.0]);
    let rotation_deg = interpolate_float(&transform.rotation, t)
        .or(transform.rotation.value)
        .unwrap_or(0.0);
    let scale = interpolate_vec2(&transform.scale, t)
        .or(transform.scale.value)
        .unwrap_or([1.0, 1.0]);
    let pivot = interpolate_vec2(&transform.pivot, t)
        .or(transform.pivot.value)
        .unwrap_or([0.0, 0.0]);
    let opacity = interpolate_float(&transform.opacity, t)
        .or(transform.opacity.value)
        .unwrap_or(1.0);

    TransformValues {
        position: [location[0], location[1]],
        rotation_deg,
        scale,
        pivot,
        opacity,
    }
}

pub(super) fn mapped_world_matrix(
    local: TransformValues,
    size: [f32; 2],
    canvas_size: [f32; 2],
    layer_index: i32,
    coord_config: CoordMappingConfig,
    parent_matrix: [f32; 16],
) -> [f32; 16] {
    let mapped = apply_coord_mapping(
        local.position,
        local.rotation_deg,
        local.scale,
        size,
        canvas_size,
        layer_index,
        &coord_config,
    );
    multiply_4x4_column_major(parent_matrix, mapped)
}

pub(super) fn base_element(
    id: i32,
    parent_id: i32,
    layer_index: i32,
    world_matrix: [f32; 16],
    local: TransformValues,
    size: [f32; 2],
    canvas_size: [f32; 2],
    start_time: i32,
    end_time: i32,
) -> FlatElement {
    FlatElement {
        id,
        parent_id,
        layer_index,
        world_matrix,
        opacity: local.opacity,
        am_position: local.position,
        am_rotation_deg: local.rotation_deg,
        am_scale: local.scale,
        am_anchor: local.pivot,
        element_width: size[0],
        element_height: size[1],
        canvas_width: canvas_size[0],
        canvas_height: canvas_size[1],
        start_time_secs: start_time as f32 / 1000.0,
        end_time_secs: end_time as f32 / 1000.0,
        ..Default::default()
    }
}

pub(super) fn fill_shape_fields(
    element: &mut FlatElement,
    fill_type: &str,
    fill_image: &str,
    fill_color: Option<&AmFillColor>,
    gradient: Option<&AmGradient>,
    t: f32,
    strings: &mut Vec<CString>,
) {
    element.fill_type = match fill_type {
        "color" => 1,
        "gradient" => gradient
            .map(|g| if g.gradient_type == "radial" { 3 } else { 2 })
            .unwrap_or(2),
        "image" | "media" => 4,
        _ => 0,
    };

    if let Some(fill_color) = fill_color {
        element.fill_color = animated_fill_color(fill_color, t).unwrap_or(element.fill_color);
    }
    if !fill_image.is_empty() {
        element.fill_image_uri = push_string(strings, fill_image);
    }
    if let Some(gradient) = gradient {
        element.fill_gradient_start = gradient.start.unwrap_or([0.0, 0.0]);
        element.fill_gradient_end = gradient.end.unwrap_or([0.0, 0.0]);
        element.fill_gradient_start_color =
            parse_color(&gradient.start_color).unwrap_or([0.0, 0.0, 0.0, 0.0]);
        element.fill_gradient_end_color =
            parse_color(&gradient.end_color).unwrap_or([0.0, 0.0, 0.0, 0.0]);
    }
}

pub(super) fn fill_stroke_fields(
    element: &mut FlatElement,
    stroke: Option<&crate::schema::AmStroke>,
) {
    let Some(stroke) = stroke else {
        return;
    };
    element.stroke_width = stroke
        .size
        .as_ref()
        .and_then(|size| {
            size.value.or_else(|| {
                size.keyframes
                    .first()
                    .and_then(|keyframe| keyframe.value.parse::<f32>().ok())
            })
        })
        .unwrap_or(if stroke.end_size > 0.0 { 4.0 } else { 0.0 });
    element.stroke_color = stroke
        .color
        .as_ref()
        .and_then(|color| parse_color(&color.value).ok())
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
    element.stroke_cap = match stroke.cap.as_str() {
        "round" => 1,
        "square" => 2,
        _ => 0,
    };
    element.stroke_join = match stroke.join.as_str() {
        "round" => 1,
        "bevel" => 2,
        _ => 0,
    };
}

pub(super) fn fill_effect_fields(element: &mut FlatElement, effects: &[AmEffect]) {
    let all_effects = crate::effects_registry::all_effects();
    for (idx, effect) in effects.iter().take(MAX_EFFECTS_PER_ELEMENT).enumerate() {
        let effect_type = all_effects
            .iter()
            .position(|def| def.id == effect.id)
            .map(|index| index as i32 + 1)
            .unwrap_or(0);
        element.effects[idx] = EffectInstance {
            effect_type,
            params: effect_params(effect),
        };
    }
    element.effects_count = effects.len().min(MAX_EFFECTS_PER_ELEMENT) as i32;
}

fn effect_params(effect: &AmEffect) -> [f32; 16] {
    let mut params = [0.0; 16];
    for (index, property) in effect.properties.iter().take(16).enumerate() {
        params[index] = property.value.parse::<f32>().unwrap_or(0.0);
    }
    params
}

fn animated_fill_color(fill_color: &AmFillColor, t: f32) -> Option<[f32; 4]> {
    let value = if fill_color.value.is_empty() {
        None
    } else {
        parse_keyframe_color(&fill_color.value)
    };
    let animated = AmAnimatedColor {
        value,
        keyframes: fill_color.keyframes.clone(),
    };
    interpolate_color(&animated, t).map(vec4_to_array)
}

pub(super) fn property_size(
    properties: &[AmProperty],
    shape_type: &str,
    t: f32,
) -> Option<([f32; 2], Vec<AmKeyframe>)> {
    for property in properties {
        if property.name != "size" || property.prop_type != "vec2" {
            continue;
        }
        let value = if property.value.is_empty() {
            property
                .keyframes
                .first()
                .and_then(|keyframe| parse_vec2(&keyframe.value).ok())
        } else {
            parse_vec2(&property.value).ok()
        }?;
        let keyframes = property.keyframes.clone();
        let sampled = if keyframes.is_empty() {
            value
        } else {
            let animated = crate::schema::AmAnimatedVec2 {
                value: Some(value),
                keyframes: keyframes.clone(),
            };
            interpolate_vec2(&animated, t).unwrap_or(value)
        };
        return Some((
            [(sampled[0] * 2.0).abs(), (sampled[1] * 2.0).abs()],
            keyframes,
        ));
    }

    infer_shape_size(properties, shape_type, t).map(|size| (size, Vec::new()))
}

pub(super) fn sample_float_property(
    properties: &[AmProperty],
    name: &str,
    default: f32,
    t: f32,
) -> f32 {
    for property in properties {
        if property.name != name || property.prop_type != "float" {
            continue;
        }
        let value = if property.value.is_empty() {
            property
                .keyframes
                .first()
                .and_then(|keyframe| keyframe.value.parse::<f32>().ok())
        } else {
            property.value.parse::<f32>().ok()
        };
        let animated = AmAnimatedFloat {
            value: value.or(Some(default)),
            keyframes: property.keyframes.clone(),
        };
        return interpolate_float(&animated, t).unwrap_or(default);
    }
    default
}

pub(super) fn sample_vec2_property(
    properties: &[AmProperty],
    name: &str,
    default: [f32; 2],
    t: f32,
) -> [f32; 2] {
    for property in properties {
        if property.name != name || property.prop_type != "vec2" {
            continue;
        }
        let value = if property.value.is_empty() {
            property
                .keyframes
                .first()
                .and_then(|keyframe| parse_vec2(&keyframe.value).ok())
        } else {
            parse_vec2(&property.value).ok()
        };
        let animated = AmAnimatedVec2 {
            value: value.or(Some(default)),
            keyframes: property.keyframes.clone(),
        };
        return interpolate_vec2(&animated, t).unwrap_or(default);
    }
    default
}

pub(super) fn fill_shape_params(
    element: &mut FlatElement,
    properties: &[AmProperty],
    shape_type: &str,
    size_keyframe_count: usize,
    t: f32,
) {
    element.shape_params[0] = element.element_width;
    element.shape_params[1] = element.element_height;
    element.shape_params[2] = size_keyframe_count as f32;
    element.shape_params[3] = match shape_type {
        ".roundrect" => sample_float_property(properties, "cornerRadius", 0.0, t),
        ".poly" => sample_float_property(properties, "sideCount", 6.0, t),
        ".star" | ".multifoil" => sample_float_property(properties, "pointCount", 5.0, t),
        ".pie" | ".arc" => sample_float_property(properties, "startAngle", 0.0, t),
        ".plus" => sample_float_property(properties, "stemSize", 50.0, t),
        ".arrow" => sample_float_property(properties, "lineWidth", 20.0, t),
        _ => 0.0,
    };
    element.shape_params[4] = match shape_type {
        ".poly" | ".pie" | ".arc" => sample_float_property(properties, "radius", 50.0, t),
        ".star" | ".multifoil" => sample_float_property(properties, "outerRadius", 50.0, t),
        ".arrow" => sample_float_property(properties, "headWidth", 40.0, t),
        _ => 0.0,
    };
    element.shape_params[5] = match shape_type {
        ".poly" | ".star" | ".multifoil" => {
            sample_float_property(properties, "offsetAngle", 0.0, t)
        }
        ".pie" | ".arc" => sample_float_property(properties, "endAngle", 270.0, t),
        ".arrow" => sample_float_property(properties, "headLength", 30.0, t),
        _ => 0.0,
    };
    element.shape_params[6] = match shape_type {
        ".star" | ".multifoil" => sample_float_property(properties, "innerRadius", 25.0, t),
        _ => 0.0,
    };

    let point_names: &[(&str, [f32; 2])] = match shape_type {
        ".line" => &[("p1", [0.0, 0.0]), ("p2", [50.0, 0.0])],
        ".arrow" => &[("start", [0.0, 0.0]), ("end", [100.0, 0.0])],
        ".triangle" => &[
            ("p1", [0.0, -50.0]),
            ("p2", [-50.0, 50.0]),
            ("p3", [50.0, 50.0]),
        ],
        ".quad" => &[
            ("p1", [-50.0, -50.0]),
            ("p2", [50.0, -50.0]),
            ("p3", [50.0, 50.0]),
            ("p4", [-50.0, 50.0]),
        ],
        ".penta" => &[
            ("p1", [0.0, -50.0]),
            ("p2", [-47.5, -15.5]),
            ("p3", [-29.4, 40.5]),
            ("p4", [29.4, 40.5]),
            ("p5", [47.5, -15.5]),
        ],
        _ => &[],
    };

    for (index, (name, default)) in point_names.iter().take(4).enumerate() {
        let point = sample_vec2_property(properties, name, *default, t);
        let base = 7 + index * 2;
        element.shape_params[base] = point[0];
        element.shape_params[base + 1] = point[1];
    }
    if point_names.len() > 4 {
        let point = sample_vec2_property(properties, point_names[4].0, point_names[4].1, t);
        element.shape_params[15] = point[0];
        element.shape_params[6] = point[1];
    }
}

fn points_bounding_size(points: &[[f32; 2]]) -> Option<[f32; 2]> {
    let first = points.first()?;
    let mut min_x = first[0];
    let mut max_x = first[0];
    let mut min_y = first[1];
    let mut max_y = first[1];
    for point in points.iter().skip(1) {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);
        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
    }
    Some([
        (max_x - min_x).abs().max(1.0),
        (max_y - min_y).abs().max(1.0),
    ])
}

fn infer_shape_size(properties: &[AmProperty], shape_type: &str, t: f32) -> Option<[f32; 2]> {
    match shape_type {
        ".poly" | ".pie" | ".arc" => {
            let radius = sample_float_property(properties, "radius", 50.0, t).abs();
            Some([radius * 2.0, radius * 2.0])
        }
        ".star" | ".multifoil" => {
            let outer_radius = sample_float_property(properties, "outerRadius", 50.0, t).abs();
            Some([outer_radius * 2.0, outer_radius * 2.0])
        }
        ".line" => {
            let p1 = sample_vec2_property(properties, "p1", [0.0, 0.0], t);
            let p2 = sample_vec2_property(properties, "p2", [50.0, 0.0], t);
            points_bounding_size(&[p1, p2])
        }
        ".triangle" => {
            let p1 = sample_vec2_property(properties, "p1", [0.0, -50.0], t);
            let p2 = sample_vec2_property(properties, "p2", [-50.0, 50.0], t);
            let p3 = sample_vec2_property(properties, "p3", [50.0, 50.0], t);
            points_bounding_size(&[p1, p2, p3])
        }
        ".quad" => {
            let p1 = sample_vec2_property(properties, "p1", [-50.0, -50.0], t);
            let p2 = sample_vec2_property(properties, "p2", [50.0, -50.0], t);
            let p3 = sample_vec2_property(properties, "p3", [50.0, 50.0], t);
            let p4 = sample_vec2_property(properties, "p4", [-50.0, 50.0], t);
            points_bounding_size(&[p1, p2, p3, p4])
        }
        ".penta" => {
            let p1 = sample_vec2_property(properties, "p1", [0.0, -50.0], t);
            let p2 = sample_vec2_property(properties, "p2", [-47.5, -15.5], t);
            let p3 = sample_vec2_property(properties, "p3", [-29.4, 40.5], t);
            let p4 = sample_vec2_property(properties, "p4", [29.4, 40.5], t);
            let p5 = sample_vec2_property(properties, "p5", [47.5, -15.5], t);
            points_bounding_size(&[p1, p2, p3, p4, p5])
        }
        ".plus" | ".arrow" => Some([100.0, 100.0]),
        _ => None,
    }
}

pub(super) fn is_layer_active(start_time: i32, end_time: i32, hidden: bool, time_ms: f32) -> bool {
    if hidden || time_ms < start_time as f32 {
        return false;
    }
    end_time <= start_time || time_ms <= end_time as f32
}

pub(super) fn normalized_layer_time(start_time: i32, end_time: i32, time_ms: f32) -> f32 {
    if end_time <= start_time {
        return 0.0;
    }
    ((time_ms - start_time as f32) / (end_time - start_time) as f32).clamp(0.0, 1.0)
}

pub(super) fn shape_kind(shape_type: &str) -> i32 {
    match shape_type {
        ".rect" => 1,
        ".circle" | ".ellipse" => 2,
        ".poly" | ".ngon" => 3,
        ".path" => 4,
        ".line" => 5,
        ".roundrect" => 11,
        ".pie" => 12,
        ".star" => 13,
        ".multifoil" => 14,
        ".arc" => 15,
        ".plus" => 16,
        ".triangle" => 17,
        ".quad" => 18,
        ".penta" => 19,
        ".arrow" => 20,
        _ => 1,
    }
}

pub(super) fn blend_mode(blending: &str) -> i32 {
    match blending {
        "" | "normal" => 0,
        "multiply" => 1,
        "darken" => 2,
        "darker-color" => 3,
        "color-burn" => 4,
        "linear-burn" => 5,
        "screen" => 6,
        "lighten" => 7,
        "lighter-color" => 8,
        "color-dodge" => 9,
        "linear-dodge" => 10,
        "overlay" => 11,
        "soft-light" => 12,
        "hard-light" => 13,
        "soft-overlay" => 14,
        "vivid-light" => 15,
        "pin-light" => 16,
        "diff" => 17,
        "exclusion" => 18,
        "subtract" => 19,
        "divide" => 20,
        "hue" => 21,
        "saturation" => 22,
        "color" => 23,
        "luminance" => 24,
        "mask" => 100,
        "exclude" => 101,
        _ => 0,
    }
}

pub(super) fn text_align(align: &str) -> i32 {
    match align {
        "center" => 1,
        "right" => 2,
        _ => 0,
    }
}

pub(super) fn parent_id(layer_parent: u64, fallback_parent_id: i32) -> i32 {
    if layer_parent == 0 {
        fallback_parent_id
    } else {
        saturating_i32(layer_parent)
    }
}

pub(super) fn saturating_i32(value: u64) -> i32 {
    value.min(i32::MAX as u64) as i32
}

fn vec4_to_array(value: Vec4) -> [f32; 4] {
    [value.x, value.y, value.z, value.w]
}

pub(super) fn push_string(strings: &mut Vec<CString>, value: &str) -> *const c_char {
    strings.push(sanitized_c_string(value));
    strings
        .last()
        .map(|value| value.as_ptr())
        .unwrap_or(ptr::null())
}

pub(super) fn sanitized_c_string(value: &str) -> CString {
    CString::new(value.replace('\0', "")).unwrap_or_else(|_| CString::new("").unwrap())
}

pub(super) fn into_raw_c_string(value: &str) -> *mut c_char {
    sanitized_c_string(value).into_raw()
}
