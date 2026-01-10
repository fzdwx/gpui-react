use std::sync::Arc;

use gpui::{App, Background, BorderStyle, Bounds, Corners, Edges, Element, ElementId, GlobalElementId, Hitbox, Hsla, InspectorElementId, IntoElement, LayoutId, PaintQuad, Path, Pixels, Rgba, Size, Style, Window, point, px};
use serde::Deserialize;

use super::{ElementStyle, ReactElement, events::{EventHandlerFlags, insert_hitbox_if_needed, register_event_handlers}};

/// Draw command types matching TypeScript definitions
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DrawCommand {
	#[serde(rename = "clear")]
	Clear { color: String },
	#[serde(rename = "fillRect")]
	FillRect { x: f32, y: f32, width: f32, height: f32, color: String },
	#[serde(rename = "circle")]
	Circle { x: f32, y: f32, radius: f32, color: String },
	#[serde(rename = "line")]
	Line { x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: String },
	#[serde(rename = "text")]
	Text { text: String, x: f32, y: f32, size: f32, color: String },
	#[serde(rename = "path")]
	Path { points: Vec<(f32, f32)>, width: f32, color: String },
}

/// Parse color string to GPUI Hsla
/// Supports "#rrggbb" and "#rgb" formats
fn parse_color(color: &str) -> Hsla {
	let color = color.trim_start_matches('#');
	let (r, g, b) = if color.len() == 6 {
		(
			u8::from_str_radix(&color[0..2], 16).unwrap_or(0),
			u8::from_str_radix(&color[2..4], 16).unwrap_or(0),
			u8::from_str_radix(&color[4..6], 16).unwrap_or(0),
		)
	} else if color.len() == 3 {
		(
			u8::from_str_radix(&color[0..1], 16).unwrap_or(0) * 17,
			u8::from_str_radix(&color[1..2], 16).unwrap_or(0) * 17,
			u8::from_str_radix(&color[2..3], 16).unwrap_or(0) * 17,
		)
	} else {
		(0, 0, 0)
	};
	Hsla::from(Rgba { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 })
}

pub struct ReactCanvasElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	#[allow(dead_code)]
	parent_style: Option<ElementStyle>,
}

pub struct CanvasLayoutState {}

pub struct CanvasPrepaintState {
	hitbox:      Option<Hitbox>,
	event_flags: EventHandlerFlags,
}

impl ReactCanvasElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style }
	}

	fn build_style(&self) -> Style {
		let es = &self.element.style;
		let mut style = Style::default();
		if let Some(width) = es.width {
			style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(width)),
			));
		}
		if let Some(height) = es.height {
			style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(height)),
			));
		}
		if let Some(bg) = es.bg_color {
			style.background = Some(gpui::Fill::Color(gpui::rgb(bg).into()));
		}
		style.position = gpui::Position::Relative;
		style
	}

	/// Parse draw commands from element style
	fn parse_draw_commands(&self) -> Vec<DrawCommand> {
		if let Some(ref draw_commands_json) = self.element.style.draw_commands {
			// draw_commands can be either a JSON string or already parsed JSON array
			let commands_value = if draw_commands_json.is_string() {
				// It's a JSON string, parse it
				if let Some(s) = draw_commands_json.as_str() {
					serde_json::from_str::<serde_json::Value>(s).ok()
				} else {
					None
				}
			} else {
				// Already a JSON value
				Some(draw_commands_json.clone())
			};

			if let Some(value) = commands_value {
				if let Ok(commands) = serde_json::from_value::<Vec<DrawCommand>>(value) {
					return commands;
				}
			}
		}
		Vec::new()
	}

	/// Execute draw commands using GPUI paint APIs
	fn execute_draw_commands(&self, bounds: Bounds<Pixels>, window: &mut Window) {
		let commands = self.parse_draw_commands();
		let origin = bounds.origin;

		for cmd in commands {
			match cmd {
				DrawCommand::Clear { color } => {
					let quad = PaintQuad {
						bounds,
						corner_radii: Corners::default(),
						background: parse_color(&color).into(),
						border_widths: Edges::default(),
						border_color: Hsla::transparent_black(),
						border_style: BorderStyle::default(),
					};
					window.paint_quad(quad);
				}
				DrawCommand::FillRect { x, y, width, height, color } => {
					let rect_bounds = Bounds {
						origin: point(origin.x + px(x), origin.y + px(y)),
						size:   Size { width: px(width), height: px(height) },
					};
					let quad = PaintQuad {
						bounds:        rect_bounds,
						corner_radii:  Corners::default(),
						background:    parse_color(&color).into(),
						border_widths: Edges::default(),
						border_color:  Hsla::transparent_black(),
						border_style:  BorderStyle::default(),
					};
					window.paint_quad(quad);
				}
				DrawCommand::Circle { x, y, radius, color } => {
					// Draw circle as a square with 50% corner radius
					let diameter = radius * 2.0;
					let circle_bounds = Bounds {
						origin: point(origin.x + px(x - radius), origin.y + px(y - radius)),
						size:   Size { width: px(diameter), height: px(diameter) },
					};
					let corner_radius = px(radius);
					let quad = PaintQuad {
						bounds:        circle_bounds,
						corner_radii:  Corners {
							top_left:     corner_radius,
							top_right:    corner_radius,
							bottom_left:  corner_radius,
							bottom_right: corner_radius,
						},
						background:    parse_color(&color).into(),
						border_widths: Edges::default(),
						border_color:  Hsla::transparent_black(),
						border_style:  BorderStyle::default(),
					};
					window.paint_quad(quad);
				}
				DrawCommand::Line { x1, y1, x2, y2, width: _, color } => {
					// Draw line using path
					let start = point(origin.x + px(x1), origin.y + px(y1));
					let end = point(origin.x + px(x2), origin.y + px(y2));
					let mut path = Path::new(start);
					path.line_to(end);
					window.paint_path(path, parse_color(&color));
				}
				DrawCommand::Text { text: _, x: _, y: _, size: _, color: _ } => {
					// Text rendering requires more complex setup with fonts
					// For now, skip text commands - they can be rendered via child elements
					log::debug!("Text draw command not yet implemented in canvas");
				}
				DrawCommand::Path { points, width: _, color } => {
					if points.len() >= 2 {
						let start = point(origin.x + px(points[0].0), origin.y + px(points[0].1));
						let mut path = Path::new(start);
						for (px_val, py_val) in points.iter().skip(1) {
							path.line_to(point(origin.x + px(*px_val), origin.y + px(*py_val)));
						}
						window.paint_path(path, parse_color(&color));
					}
				}
			}
		}
	}
}

impl Element for ReactCanvasElement {
	type PrepaintState = CanvasPrepaintState;
	type RequestLayoutState = CanvasLayoutState;

	fn id(&self) -> Option<ElementId> { Some(ElementId::Integer(self.element.global_id)) }

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let style = self.build_style();
		// Canvas doesn't have layout children - it draws via commands
		let layout_id = window.request_layout(style, std::iter::empty(), cx);
		(layout_id, CanvasLayoutState {})
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		_cx: &mut App,
	) -> Self::PrepaintState {
		let event_flags = EventHandlerFlags::from_handlers(
			self.element.event_handlers.as_ref(),
			self.element.style.tab_index,
		);
		let hitbox = insert_hitbox_if_needed(&event_flags, bounds, window);
		CanvasPrepaintState { hitbox, event_flags }
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		_cx: &mut App,
	) {
		let element_id = self.element.global_id;
		let window_id = self.window_id;

		// Paint background first if specified
		if let Some(bg) = self.element.style.bg_color {
			let bg_color = gpui::rgb(bg);
			let quad = PaintQuad {
				bounds,
				corner_radii: Corners::default(),
				background: Hsla::from(bg_color).into(),
				border_widths: Edges::default(),
				border_color: Hsla::transparent_black(),
				border_style: BorderStyle::default(),
			};
			window.paint_quad(quad);
		}

		// Execute draw commands
		self.execute_draw_commands(bounds, window);

		// Register event handlers
		register_event_handlers(
			&prepaint.event_flags,
			prepaint.hitbox.as_ref(),
			window_id,
			element_id,
			window,
		);
	}
}

impl IntoElement for ReactCanvasElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}
