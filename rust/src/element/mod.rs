use std::sync::Arc;

use gpui::{AlignContent, AlignItems, AlignSelf, AnyElement, BoxShadow, Fill, FlexDirection, FlexWrap, Hsla, InteractiveElement, IntoElement, JustifyContent, Overflow, ParentElement, Position, Rgba, Style, point, px, rgb};
use serde_json::Value;

pub mod div;
pub mod img;
pub mod span;
pub mod text;

pub use div::ReactDivElement;
pub use img::ReactImgElement;
pub use span::ReactSpanElement;
pub use text::ReactTextElement;

/// Pre-computed element kind to avoid string matching every frame
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElementKind {
	Div,
	Span,
	Text,
	Img,
	Unknown,
}

impl ElementKind {
	pub fn from_str(s: &str) -> Self {
		match s {
			"div" => ElementKind::Div,
			"span" => ElementKind::Span,
			"text" => ElementKind::Text,
			"img" => ElementKind::Img,
			_ => ElementKind::Unknown,
		}
	}
}

#[derive(Clone)]
pub struct ReactElement {
	pub global_id:         u64,
	pub element_type:      String,
	pub element_kind:      ElementKind, // Pre-computed for fast dispatch
	pub text:              Option<String>,
	pub children:          Vec<Arc<ReactElement>>,
	pub style:             ElementStyle,
	pub event_handlers:    Option<serde_json::Value>,
	/// Cached GPUI Style to avoid recomputing every frame
	pub cached_gpui_style: Option<Style>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ElementStyle {
	// Text properties (inheritable)
	pub text_color:     Option<u32>,
	pub text_size:      Option<f32>,
	pub font_weight:    Option<u32>, // 100-900
	pub font_family:    Option<String>,
	pub line_height:    Option<f32>,
	pub text_align:     Option<String>, // "left", "center", "right"
	pub letter_spacing: Option<f32>,

	// Other inheritable properties
	pub cursor:     Option<String>,
	pub visibility: Option<String>, // "visible", "hidden"

	// Non-inheritable properties
	pub bg_color: Option<u32>,
	pub width:    Option<f32>,
	pub height:   Option<f32>,

	// Size constraints
	pub min_width:    Option<f32>,
	pub max_width:    Option<f32>,
	pub min_height:   Option<f32>,
	pub max_height:   Option<f32>,
	pub aspect_ratio: Option<f32>,

	// Margin
	pub margin_top:    Option<f32>,
	pub margin_right:  Option<f32>,
	pub margin_bottom: Option<f32>,
	pub margin_left:   Option<f32>,

	// Padding
	pub padding_top:    Option<f32>,
	pub padding_right:  Option<f32>,
	pub padding_bottom: Option<f32>,
	pub padding_left:   Option<f32>,

	// Position
	pub position: Option<String>, // "relative", "absolute"
	pub top:      Option<f32>,
	pub right:    Option<f32>,
	pub bottom:   Option<f32>,
	pub left:     Option<f32>,

	// Overflow
	pub overflow_x: Option<String>, // "visible", "hidden", "scroll", "clip"
	pub overflow_y: Option<String>,

	// Border widths (4 sides)
	pub border_top_width:    Option<f32>,
	pub border_right_width:  Option<f32>,
	pub border_bottom_width: Option<f32>,
	pub border_left_width:   Option<f32>,
	pub border_style:        Option<String>, // "solid", "dashed"
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
	pub flex_wrap:       Option<String>, // "nowrap", "wrap", "wrap-reverse"
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
	pub opacity: Option<f32>,
	pub src:     Option<String>,
	pub alt:     Option<String>,

	// Hover style
	pub hover_style: Option<Box<ElementStyle>>,
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
		if self.text_color.is_none() {
			self.text_color = parent.text_color;
		}
		if self.text_size.is_none() {
			self.text_size = parent.text_size;
		}
		if self.font_weight.is_none() {
			self.font_weight = parent.font_weight;
		}
		if self.font_family.is_none() {
			self.font_family = parent.font_family.clone();
		}
		if self.line_height.is_none() {
			self.line_height = parent.line_height;
		}
		if self.text_align.is_none() {
			self.text_align = parent.text_align.clone();
		}
		if self.letter_spacing.is_none() {
			self.letter_spacing = parent.letter_spacing;
		}
		// Other inheritable
		if self.cursor.is_none() {
			self.cursor = parent.cursor.clone();
		}
		if self.visibility.is_none() {
			self.visibility = parent.visibility.clone();
		}
	}

	/// Build GPUI Style from ElementStyle
	/// `default_bg` - Optional default background color (div uses Some(0x2d2d2d),
	/// span uses None)
	pub fn build_gpui_style(&self, default_bg: Option<u32>) -> Style {
		let mut style = Style::default();

		self.apply_display_flex(&mut style);
		self.apply_positioning(&mut style);
		self.apply_sizing(&mut style);
		self.apply_spacing(&mut style);
		self.apply_overflow(&mut style);
		self.apply_borders(&mut style);
		self.apply_box_shadow(&mut style);
		self.apply_visual_effects(&mut style, default_bg);

		style
	}

	/// Apply display and flexbox properties
	fn apply_display_flex(&self, style: &mut Style) {
		// Display and flex
		if self.display.as_ref().map(|s| s.as_str()) == Some("flex") {
			style.display = gpui::Display::Flex;
			style.flex_direction = FlexDirection::Row;
		}

		// Flex direction
		match self.flex_direction.as_ref().map(|s| s.as_str()) {
			Some("row") => style.flex_direction = FlexDirection::Row,
			Some("row-reverse") => style.flex_direction = FlexDirection::RowReverse,
			Some("column") => style.flex_direction = FlexDirection::Column,
			Some("column-reverse") => style.flex_direction = FlexDirection::ColumnReverse,
			_ => {}
		}

		// Flex wrap
		match self.flex_wrap.as_ref().map(|s| s.as_str()) {
			Some("wrap") => style.flex_wrap = FlexWrap::Wrap,
			Some("wrap-reverse") => style.flex_wrap = FlexWrap::WrapReverse,
			Some("nowrap") => style.flex_wrap = FlexWrap::NoWrap,
			_ => {}
		}

		// Flex grow/shrink/basis
		if let Some(grow) = self.flex_grow {
			style.flex_grow = grow;
		}
		if let Some(shrink) = self.flex_shrink {
			style.flex_shrink = shrink;
		}
		if let Some(basis) = self.flex_basis {
			style.flex_basis = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(basis)),
			));
		}

		// Justify content
		match self.justify_content.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => style.justify_content = Some(JustifyContent::FlexStart),
			Some("center") => style.justify_content = Some(JustifyContent::Center),
			Some("flex-end") => style.justify_content = Some(JustifyContent::FlexEnd),
			Some("space-between") => style.justify_content = Some(JustifyContent::SpaceBetween),
			Some("space-around") => style.justify_content = Some(JustifyContent::SpaceAround),
			Some("space-evenly") => style.justify_content = Some(JustifyContent::SpaceEvenly),
			_ => {}
		}

		// Align items
		match self.align_items.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => style.align_items = Some(AlignItems::FlexStart),
			Some("center") => style.align_items = Some(AlignItems::Center),
			Some("flex-end") => style.align_items = Some(AlignItems::FlexEnd),
			Some("stretch") => style.align_items = Some(AlignItems::Stretch),
			Some("baseline") => style.align_items = Some(AlignItems::Baseline),
			_ => {}
		}

		// Align self
		match self.align_self.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => style.align_self = Some(AlignSelf::FlexStart),
			Some("center") => style.align_self = Some(AlignSelf::Center),
			Some("flex-end") => style.align_self = Some(AlignSelf::FlexEnd),
			Some("stretch") => style.align_self = Some(AlignSelf::Stretch),
			Some("baseline") => style.align_self = Some(AlignSelf::Baseline),
			_ => {}
		}

		// Align content
		match self.align_content.as_ref().map(|s| s.as_str()) {
			Some("flex-start") => style.align_content = Some(AlignContent::FlexStart),
			Some("center") => style.align_content = Some(AlignContent::Center),
			Some("flex-end") => style.align_content = Some(AlignContent::FlexEnd),
			Some("space-between") => style.align_content = Some(AlignContent::SpaceBetween),
			Some("space-around") => style.align_content = Some(AlignContent::SpaceAround),
			Some("stretch") => style.align_content = Some(AlignContent::Stretch),
			_ => {}
		}
	}

	/// Apply position and inset properties
	fn apply_positioning(&self, style: &mut Style) {
		// Position type
		match self.position.as_ref().map(|s| s.as_str()) {
			Some("absolute") => style.position = Position::Absolute,
			Some("relative") => style.position = Position::Relative,
			_ => {}
		}

		// Inset (top, right, bottom, left)
		if let Some(top) = self.top {
			style.inset.top = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(top)),
			));
		}
		if let Some(right) = self.right {
			style.inset.right = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(right)),
			));
		}
		if let Some(bottom) = self.bottom {
			style.inset.bottom = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(bottom)),
			));
		}
		if let Some(left) = self.left {
			style.inset.left = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(left)),
			));
		}
	}

	/// Apply width, height, and size constraints
	fn apply_sizing(&self, style: &mut Style) {
		// Size
		if let Some(width) = self.width {
			style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(width)),
			));
		}
		if let Some(height) = self.height {
			style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(height)),
			));
		}

		// Min/max size
		if let Some(min_w) = self.min_width {
			style.min_size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(min_w)),
			));
		}
		if let Some(max_w) = self.max_width {
			style.max_size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(max_w)),
			));
		}
		if let Some(min_h) = self.min_height {
			style.min_size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(min_h)),
			));
		}
		if let Some(max_h) = self.max_height {
			style.max_size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(max_h)),
			));
		}

		// Aspect ratio
		if let Some(ratio) = self.aspect_ratio {
			style.aspect_ratio = Some(ratio);
		}
	}

	/// Apply padding, margin, and gap properties
	fn apply_spacing(&self, style: &mut Style) {
		// Padding
		if let Some(pt) = self.padding_top {
			style.padding.top = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pt)));
		}
		if let Some(pr) = self.padding_right {
			style.padding.right = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pr)));
		}
		if let Some(pb) = self.padding_bottom {
			style.padding.bottom = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pb)));
		}
		if let Some(pl) = self.padding_left {
			style.padding.left = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(pl)));
		}

		// Margin
		if let Some(mt) = self.margin_top {
			style.margin.top = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(mt)),
			));
		}
		if let Some(mr) = self.margin_right {
			style.margin.right = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(mr)),
			));
		}
		if let Some(mb) = self.margin_bottom {
			style.margin.bottom = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(mb)),
			));
		}
		if let Some(ml) = self.margin_left {
			style.margin.left = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(ml)),
			));
		}

		// Gap
		if let Some(gap) = self.gap {
			style.gap.width = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(gap)));
			style.gap.height = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(gap)));
		}
		if let Some(row_gap) = self.row_gap {
			style.gap.height = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(row_gap)));
		}
		if let Some(col_gap) = self.column_gap {
			style.gap.width = gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(px(col_gap)));
		}
	}

	/// Apply overflow properties
	fn apply_overflow(&self, style: &mut Style) {
		if let Some(ref ox) = self.overflow_x {
			style.overflow.x = match ox.as_str() {
				"hidden" => Overflow::Hidden,
				"scroll" => Overflow::Scroll,
				"clip" => Overflow::Clip,
				_ => Overflow::Visible,
			};
		}
		if let Some(ref oy) = self.overflow_y {
			style.overflow.y = match oy.as_str() {
				"hidden" => Overflow::Hidden,
				"scroll" => Overflow::Scroll,
				"clip" => Overflow::Clip,
				_ => Overflow::Visible,
			};
		}
	}

	/// Apply border width, color, and radius properties
	fn apply_borders(&self, style: &mut Style) {
		// Border widths
		if let Some(w) = self.border_top_width {
			style.border_widths.top = gpui::AbsoluteLength::Pixels(px(w));
		}
		if let Some(w) = self.border_right_width {
			style.border_widths.right = gpui::AbsoluteLength::Pixels(px(w));
		}
		if let Some(w) = self.border_bottom_width {
			style.border_widths.bottom = gpui::AbsoluteLength::Pixels(px(w));
		}
		if let Some(w) = self.border_left_width {
			style.border_widths.left = gpui::AbsoluteLength::Pixels(px(w));
		}

		// Border color
		let border_color = self.border_color.map(|c| rgb(c).into());
		if border_color.is_some()
			|| self.border_top_width.is_some()
			|| self.border_right_width.is_some()
			|| self.border_bottom_width.is_some()
			|| self.border_left_width.is_some()
		{
			style.border_color = border_color.or(Some(rgb(0x808080).into()));
		}

		// Border radius
		if let Some(radius) = self.border_radius {
			let r = gpui::AbsoluteLength::Pixels(px(radius));
			style.corner_radii.top_left = r;
			style.corner_radii.top_right = r;
			style.corner_radii.bottom_left = r;
			style.corner_radii.bottom_right = r;
		}
	}

	/// Apply box shadow properties
	fn apply_box_shadow(&self, style: &mut Style) {
		if self.box_shadow_color.is_some()
			|| self.box_shadow_blur.is_some()
			|| self.box_shadow_offset_x.is_some()
			|| self.box_shadow_offset_y.is_some()
		{
			let color = self.box_shadow_color.unwrap_or(0x000000);
			let (r, g, b) = ((color >> 16) & 0xff, (color >> 8) & 0xff, color & 0xff);
			style.box_shadow = vec![BoxShadow {
				color:         Hsla::from(Rgba {
					r: r as f32 / 255.0,
					g: g as f32 / 255.0,
					b: b as f32 / 255.0,
					a: 0.5,
				}),
				offset:        point(
					px(self.box_shadow_offset_x.unwrap_or(0.0)),
					px(self.box_shadow_offset_y.unwrap_or(0.0)),
				),
				blur_radius:   px(self.box_shadow_blur.unwrap_or(0.0)),
				spread_radius: px(self.box_shadow_spread.unwrap_or(0.0)),
			}];
		}
	}

	/// Apply background, opacity, and other visual effects
	fn apply_visual_effects(&self, style: &mut Style, default_bg: Option<u32>) {
		// Background
		if let Some(bg) = self.bg_color {
			style.background = Some(Fill::Color(rgb(bg).into()));
		} else if let Some(default) = default_bg {
			style.background = Some(Fill::Color(rgb(default).into()));
		}

		// Opacity
		if let Some(opacity) = self.opacity {
			style.opacity = Some(opacity);
		}
	}

	/// Check if overflow clipping should be applied
	pub fn should_clip(&self) -> bool {
		matches!(self.overflow_x.as_ref().map(|s| s.as_str()), Some("hidden") | Some("clip"))
			|| matches!(self.overflow_y.as_ref().map(|s| s.as_str()), Some("hidden") | Some("clip"))
	}
}

/// Paint children with optional overflow clipping
/// This helper function reduces code duplication across element types
pub fn paint_children_with_clip<F>(
	children: &mut [AnyElement],
	bounds: gpui::Bounds<gpui::Pixels>,
	should_clip: bool,
	window: &mut gpui::Window,
	cx: &mut gpui::App,
	mut paint_child: F,
) where
	F: FnMut(&mut AnyElement, &mut gpui::Window, &mut gpui::App),
{
	use gpui::ContentMask;

	if should_clip {
		let mask = ContentMask { bounds };
		window.with_content_mask(Some(mask), |window| {
			for child in children.iter_mut() {
				paint_child(child, window, cx);
			}
		});
	} else {
		for child in children.iter_mut() {
			paint_child(child, window, cx);
		}
	}
}

/// Create a new element that implements Element trait directly
/// Uses pre-computed ElementKind for fast dispatch (no string matching)
pub fn create_element(
	element: Arc<ReactElement>,
	window_id: u64,
	parent_style: Option<ElementStyle>,
) -> AnyElement {
	match element.element_kind {
		ElementKind::Div => ReactDivElement::new(element, window_id, parent_style).into_any_element(),
		ElementKind::Span => ReactSpanElement::new(element, window_id, parent_style).into_any_element(),
		ElementKind::Text => ReactTextElement::new(element, window_id, parent_style).into_any_element(),
		ElementKind::Img => ReactImgElement::new(element, window_id, parent_style).into_any_element(),
		ElementKind::Unknown => gpui::div()
			.id(element.global_id as usize)
			.child(format!("[Unknown: {}]", element.element_type))
			.into_any_element(),
	}
}
