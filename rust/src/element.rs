use std::sync::Arc;

use serde_json::Value;

#[derive(Clone, PartialEq)]
pub struct ReactElement {
	pub global_id:      u64,
	pub element_type:   String,
	pub text:           Option<String>,
	pub children:       Vec<Arc<ReactElement>>,
	pub style:          ElementStyle,
	pub event_handlers: Option<serde_json::Value>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ElementStyle {
	pub text_color:      Option<u32>,
	pub bg_color:        Option<u32>,
	pub border_color:    Option<u32>,
	pub text_size:       Option<f32>,
	pub width:           Option<f32>,
	pub height:          Option<f32>,
	pub margin_top:      Option<f32>,
	pub margin_right:    Option<f32>,
	pub margin_bottom:   Option<f32>,
	pub margin_left:     Option<f32>,
	pub padding_top:     Option<f32>,
	pub padding_right:   Option<f32>,
	pub padding_bottom:  Option<f32>,
	pub padding_left:    Option<f32>,
	pub display:         Option<String>,
	pub flex_direction:  Option<String>,
	pub justify_content: Option<String>,
	pub align_items:     Option<String>,
	pub gap:             Option<f32>,
	pub border_radius:   Option<f32>,
	pub opacity:         Option<f32>,
	pub src:             Option<String>,
	pub alt:             Option<String>,
}

impl ElementStyle {
	#[rustfmt::skip]
	pub fn from_json(style_obj: &Value) -> Self {
        ElementStyle {
            text_color: style_obj.get("textColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            bg_color: style_obj.get("bgColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_color: style_obj.get("borderColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            text_size: style_obj.get("textSize").and_then(|v| v.as_f64()).map(|v| v as f32),
            width: style_obj.get("width").and_then(|v| v.as_f64()).map(|v| v as f32),
            height: style_obj.get("height").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_top: style_obj.get("marginTop").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_right: style_obj.get("marginRight").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_bottom: style_obj.get("marginBottom").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_left: style_obj.get("marginLeft").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_top: style_obj.get("paddingTop").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_right: style_obj.get("paddingRight").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_bottom: style_obj.get("paddingBottom").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_left: style_obj.get("paddingLeft").and_then(|v| v.as_f64()).map(|v| v as f32),
            display: style_obj.get("display").and_then(|v| v.as_str()).map(|s| s.to_string()),
            flex_direction: style_obj.get("flexDirection").and_then(|v| v.as_str()).map(|s| s.to_string()),
            justify_content: style_obj.get("justifyContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
            align_items: style_obj.get("alignItems").and_then(|v| v.as_str()).map(|s| s.to_string()),
            gap: style_obj.get("gap").and_then(|v| v.as_f64()).map(|v| v as f32),
            border_radius: style_obj.get("borderRadius").and_then(|v| v.as_f64()).map(|v| v as f32),
            opacity: style_obj.get("opacity").and_then(|v| v.as_f64()).map(|v| v as f32),
            src: style_obj.get("src").and_then(|v| v.as_str()).map(|s| s.to_string()),
            alt: style_obj.get("alt").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
}

pub type EventId = u64;
