use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct ReactElement {
    pub global_id: u64,
    pub element_type: String,
    pub text: Option<String>,
    pub children: Vec<Arc<ReactElement>>,
    pub style: ElementStyle,
    pub event_handlers: Option<serde_json::Value>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ElementStyle {
    pub text_color: Option<u32>,
    pub bg_color: Option<u32>,
    pub border_color: Option<u32>,
    pub text_size: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub display: Option<String>,
    pub flex_direction: Option<String>,
    pub justify_content: Option<String>,
    pub align_items: Option<String>,
    pub gap: Option<f32>,
    pub border_radius: Option<f32>,
    pub opacity: Option<f32>,
    pub src: Option<String>,
    pub alt: Option<String>,
}

pub type EventId = u64;
