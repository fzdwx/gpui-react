use std::sync::Arc;

use gpui::{AnyElement, InteractiveElement, IntoElement, ParentElement};
use serde_json::Value;

pub mod div;
pub mod img;
pub mod span;
pub mod text;

pub use div::ReactDivElement;
pub use img::ReactImgElement;
pub use span::ReactSpanElement;
pub use text::ReactTextElement;

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
	// Text properties (inheritable)
	pub text_color:      Option<u32>,
	pub text_size:       Option<f32>,
	pub font_weight:     Option<u32>,      // 100-900
	pub font_family:     Option<String>,
	pub line_height:     Option<f32>,
	pub text_align:      Option<String>,   // "left", "center", "right"
	pub letter_spacing:  Option<f32>,

	// Other inheritable properties
	pub cursor:          Option<String>,
	pub visibility:      Option<String>,   // "visible", "hidden"

	// Non-inheritable properties
	pub bg_color:        Option<u32>,
	pub width:           Option<f32>,
	pub height:          Option<f32>,

	// Size constraints
	pub min_width:       Option<f32>,
	pub max_width:       Option<f32>,
	pub min_height:      Option<f32>,
	pub max_height:      Option<f32>,
	pub aspect_ratio:    Option<f32>,

	// Margin
	pub margin_top:      Option<f32>,
	pub margin_right:    Option<f32>,
	pub margin_bottom:   Option<f32>,
	pub margin_left:     Option<f32>,

	// Padding
	pub padding_top:     Option<f32>,
	pub padding_right:   Option<f32>,
	pub padding_bottom:  Option<f32>,
	pub padding_left:    Option<f32>,

	// Position
	pub position:        Option<String>,   // "relative", "absolute"
	pub top:             Option<f32>,
	pub right:           Option<f32>,
	pub bottom:          Option<f32>,
	pub left:            Option<f32>,

	// Overflow
	pub overflow_x:      Option<String>,   // "visible", "hidden", "scroll", "clip"
	pub overflow_y:      Option<String>,

	// Border widths (4 sides)
	pub border_top_width:    Option<f32>,
	pub border_right_width:  Option<f32>,
	pub border_bottom_width: Option<f32>,
	pub border_left_width:   Option<f32>,
	pub border_style:        Option<String>,  // "solid", "dashed"
	pub border_color:        Option<u32>,
	pub border_top_color:    Option<u32>,
	pub border_right_color:  Option<u32>,
	pub border_bottom_color: Option<u32>,
	pub border_left_color:   Option<u32>,
	pub border_radius:       Option<f32>,

	// Box shadow
	pub box_shadow_offset_x: Option<f32>,
	pub box_shadow_offset_y: Option<f32>,
	pub box_shadow_blur:     Option<f32>,
	pub box_shadow_spread:   Option<f32>,
	pub box_shadow_color:    Option<u32>,

	// Flexbox
	pub display:         Option<String>,
	pub flex_direction:  Option<String>,
	pub flex_wrap:       Option<String>,   // "nowrap", "wrap", "wrap-reverse"
	pub flex_grow:       Option<f32>,
	pub flex_shrink:     Option<f32>,
	pub flex_basis:      Option<f32>,
	pub justify_content: Option<String>,
	pub align_items:     Option<String>,
	pub align_self:      Option<String>,
	pub align_content:   Option<String>,
	pub gap:             Option<f32>,
	pub row_gap:         Option<f32>,
	pub column_gap:      Option<f32>,

	// Other
	pub opacity:         Option<f32>,
	pub src:             Option<String>,
	pub alt:             Option<String>,

	// Hover style
	pub hover_style:     Option<Box<ElementStyle>>,
}

impl ElementStyle {
	#[rustfmt::skip]
	pub fn from_json(style_obj: &Value) -> Self {
        // Parse hover style recursively
        let hover_style = style_obj.get("hoverStyle")
            .and_then(|v| v.as_object())
            .map(|obj| Box::new(Self::from_json(&Value::Object(obj.clone()))));

        ElementStyle {
            // Text properties (inheritable)
            text_color: style_obj.get("textColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            text_size: style_obj.get("textSize").and_then(|v| v.as_f64()).map(|v| v as f32),
            font_weight: style_obj.get("fontWeight").and_then(|v| v.as_u64()).map(|v| v as u32),
            font_family: style_obj.get("fontFamily").and_then(|v| v.as_str()).map(|s| s.to_string()),
            line_height: style_obj.get("lineHeight").and_then(|v| v.as_f64()).map(|v| v as f32),
            text_align: style_obj.get("textAlign").and_then(|v| v.as_str()).map(|s| s.to_string()),
            letter_spacing: style_obj.get("letterSpacing").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Other inheritable
            cursor: style_obj.get("cursor").and_then(|v| v.as_str()).map(|s| s.to_string()),
            visibility: style_obj.get("visibility").and_then(|v| v.as_str()).map(|s| s.to_string()),

            // Non-inheritable
            bg_color: style_obj.get("bgColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            width: style_obj.get("width").and_then(|v| v.as_f64()).map(|v| v as f32),
            height: style_obj.get("height").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Size constraints
            min_width: style_obj.get("minWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            max_width: style_obj.get("maxWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            min_height: style_obj.get("minHeight").and_then(|v| v.as_f64()).map(|v| v as f32),
            max_height: style_obj.get("maxHeight").and_then(|v| v.as_f64()).map(|v| v as f32),
            aspect_ratio: style_obj.get("aspectRatio").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Margin
            margin_top: style_obj.get("marginTop").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_right: style_obj.get("marginRight").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_bottom: style_obj.get("marginBottom").and_then(|v| v.as_f64()).map(|v| v as f32),
            margin_left: style_obj.get("marginLeft").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Padding
            padding_top: style_obj.get("paddingTop").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_right: style_obj.get("paddingRight").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_bottom: style_obj.get("paddingBottom").and_then(|v| v.as_f64()).map(|v| v as f32),
            padding_left: style_obj.get("paddingLeft").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Position
            position: style_obj.get("position").and_then(|v| v.as_str()).map(|s| s.to_string()),
            top: style_obj.get("top").and_then(|v| v.as_f64()).map(|v| v as f32),
            right: style_obj.get("right").and_then(|v| v.as_f64()).map(|v| v as f32),
            bottom: style_obj.get("bottom").and_then(|v| v.as_f64()).map(|v| v as f32),
            left: style_obj.get("left").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Overflow
            overflow_x: style_obj.get("overflowX").and_then(|v| v.as_str()).map(|s| s.to_string()),
            overflow_y: style_obj.get("overflowY").and_then(|v| v.as_str()).map(|s| s.to_string()),

            // Border widths
            border_top_width: style_obj.get("borderTopWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            border_right_width: style_obj.get("borderRightWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            border_bottom_width: style_obj.get("borderBottomWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            border_left_width: style_obj.get("borderLeftWidth").and_then(|v| v.as_f64()).map(|v| v as f32),
            border_style: style_obj.get("borderStyle").and_then(|v| v.as_str()).map(|s| s.to_string()),
            border_color: style_obj.get("borderColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_top_color: style_obj.get("borderTopColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_right_color: style_obj.get("borderRightColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_bottom_color: style_obj.get("borderBottomColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_left_color: style_obj.get("borderLeftColor").and_then(|v| v.as_u64()).map(|v| v as u32),
            border_radius: style_obj.get("borderRadius").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Box shadow
            box_shadow_offset_x: style_obj.get("boxShadowOffsetX").and_then(|v| v.as_f64()).map(|v| v as f32),
            box_shadow_offset_y: style_obj.get("boxShadowOffsetY").and_then(|v| v.as_f64()).map(|v| v as f32),
            box_shadow_blur: style_obj.get("boxShadowBlur").and_then(|v| v.as_f64()).map(|v| v as f32),
            box_shadow_spread: style_obj.get("boxShadowSpread").and_then(|v| v.as_f64()).map(|v| v as f32),
            box_shadow_color: style_obj.get("boxShadowColor").and_then(|v| v.as_u64()).map(|v| v as u32),

            // Flexbox
            display: style_obj.get("display").and_then(|v| v.as_str()).map(|s| s.to_string()),
            flex_direction: style_obj.get("flexDirection").and_then(|v| v.as_str()).map(|s| s.to_string()),
            flex_wrap: style_obj.get("flexWrap").and_then(|v| v.as_str()).map(|s| s.to_string()),
            flex_grow: style_obj.get("flexGrow").and_then(|v| v.as_f64()).map(|v| v as f32),
            flex_shrink: style_obj.get("flexShrink").and_then(|v| v.as_f64()).map(|v| v as f32),
            flex_basis: style_obj.get("flexBasis").and_then(|v| v.as_f64()).map(|v| v as f32),
            justify_content: style_obj.get("justifyContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
            align_items: style_obj.get("alignItems").and_then(|v| v.as_str()).map(|s| s.to_string()),
            align_self: style_obj.get("alignSelf").and_then(|v| v.as_str()).map(|s| s.to_string()),
            align_content: style_obj.get("alignContent").and_then(|v| v.as_str()).map(|s| s.to_string()),
            gap: style_obj.get("gap").and_then(|v| v.as_f64()).map(|v| v as f32),
            row_gap: style_obj.get("rowGap").and_then(|v| v.as_f64()).map(|v| v as f32),
            column_gap: style_obj.get("columnGap").and_then(|v| v.as_f64()).map(|v| v as f32),

            // Other
            opacity: style_obj.get("opacity").and_then(|v| v.as_f64()).map(|v| v as f32),
            src: style_obj.get("src").and_then(|v| v.as_str()).map(|s| s.to_string()),
            alt: style_obj.get("alt").and_then(|v| v.as_str()).map(|s| s.to_string()),

            // Hover style
            hover_style,
        }
    }

	/// Inherit all inheritable CSS properties from parent
	/// This follows CSS inheritance rules where text/font properties cascade down
	pub fn inherit_from(&mut self, parent: &ElementStyle) {
		// Text properties
		if self.text_color.is_none() { self.text_color = parent.text_color; }
		if self.text_size.is_none() { self.text_size = parent.text_size; }
		if self.font_weight.is_none() { self.font_weight = parent.font_weight; }
		if self.font_family.is_none() { self.font_family = parent.font_family.clone(); }
		if self.line_height.is_none() { self.line_height = parent.line_height; }
		if self.text_align.is_none() { self.text_align = parent.text_align.clone(); }
		if self.letter_spacing.is_none() { self.letter_spacing = parent.letter_spacing; }
		// Other inheritable
		if self.cursor.is_none() { self.cursor = parent.cursor.clone(); }
		if self.visibility.is_none() { self.visibility = parent.visibility.clone(); }
	}
}

/// Create a new element that implements Element trait directly
/// This is the new optimized path that avoids recreating gpui::Div every frame
pub fn create_element(
	element: Arc<ReactElement>,
	window_id: u64,
	parent_style: Option<ElementStyle>,
) -> AnyElement {
	match element.element_type.as_str() {
		"div" => {
			ReactDivElement::new(element, window_id, parent_style).into_any_element()
		}
		"span" => {
			// span uses specialized ReactSpanElement (no default background)
			ReactSpanElement::new(element, window_id, parent_style).into_any_element()
		}
		"text" => {
			// text uses specialized lightweight ReactTextElement
			ReactTextElement::new(element, window_id, parent_style).into_any_element()
		}
		"img" => {
			// img uses specialized ReactImgElement for images
			ReactImgElement::new(element, window_id, parent_style).into_any_element()
		}
		_ => gpui::div()
			.id(element.global_id as usize)
			.child(format!("[Unknown: {}]", element.element_type))
			.into_any_element(),
	}
}
