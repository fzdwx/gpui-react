use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::element::ReactElement;

/// 窗口状态结构体 - 集中管理单个窗口的所有状态
pub struct WindowState {
    /// 根元素 ID
    pub root_element_id: AtomicU64,

    /// 元素映射表 - 存储窗口中的所有元素
    pub element_map: Mutex<HashMap<u64, Arc<ReactElement>>>,

    /// 元素树 - 窗口的渲染树
    pub element_tree: Arc<Mutex<Option<Arc<ReactElement>>>>,

    /// 渲染触发器 - 触发窗口重新渲染
    pub render_trigger: Arc<AtomicU64>,

    /// 渲染计数
    pub render_count: AtomicU64,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            root_element_id: AtomicU64::new(0),
            element_map: Mutex::new(HashMap::new()),
            element_tree: Arc::new(Mutex::new(None)),
            render_trigger: Arc::new(AtomicU64::new(0)),
            render_count: AtomicU64::new(0),
        }
    }

    /// 获取根元素 ID
    pub fn get_root_element_id(&self) -> u64 {
        self.root_element_id.load(Ordering::SeqCst)
    }

    /// 设置根元素 ID
    pub fn set_root_element_id(&self, id: u64) {
        self.root_element_id.store(id, Ordering::SeqCst);
    }

    /// 触发渲染
    pub fn trigger_render(&self) {
        self.render_trigger.fetch_add(1, Ordering::SeqCst);
    }

    /// 获取渲染触发器值
    pub fn get_render_trigger(&self) -> u64 {
        self.render_trigger.load(Ordering::SeqCst)
    }

    /// 增加渲染计数
    pub fn increment_render_count(&self) -> u64 {
        self.render_count.fetch_add(1, Ordering::SeqCst)
    }

    /// 获取根元素
    pub fn get_root_element(&self) -> Arc<ReactElement> {
        let root_id = self.get_root_element_id();

        let element_map = self
            .element_map
            .lock()
            .expect("Failed to acquire element_map lock in get_root_element");

        if root_id == 0 {
            return element_map.values().next().cloned().unwrap_or_else(|| {
                Arc::new(ReactElement {
                    global_id: 0,
                    element_type: "empty".to_string(),
                    text: None,
                    children: Vec::new(),
                    style: crate::element::ElementStyle::default(),
                    event_handlers: None,
                })
            });
        }

        element_map.get(&root_id).cloned().unwrap_or_else(|| {
            Arc::new(ReactElement {
                global_id: 0,
                element_type: "empty".to_string(),
                text: None,
                children: Vec::new(),
                style: crate::element::ElementStyle::default(),
                event_handlers: None,
            })
        })
    }

    /// 重建元素树
    pub fn rebuild_tree(&self, root_id: u64, children: &[u64]) {
        log::debug!("rebuild_tree: root_id={}, children={:?}", root_id, children);
        let element_map = self
            .element_map
            .lock()
            .expect("Failed to acquire element_map lock in rebuild_tree (first lock)");

        log::trace!("  element_map has {} entries", element_map.len());
        for (id, elem) in element_map.iter() {
            log::trace!("    id={}, type={}", id, elem.element_type);
        }

        if let Some(root) = element_map.get(&root_id) {
            log::debug!(
                "  found root element: id={}, type={}",
                root.global_id,
                root.element_type
            );
            let child_elements: Vec<Arc<ReactElement>> = children
                .iter()
                .filter_map(|id| {
                    log::trace!("    looking up child id={}", id);
                    element_map.get(id).cloned()
                })
                .collect();
            log::debug!("  found {} child elements", child_elements.len());

            drop(element_map);

            let mut element_map = self
                .element_map
                .lock()
                .expect("Failed to acquire element_map lock in rebuild_tree (second lock)");
            if let Some(root) = element_map.get_mut(&root_id) {
                log::debug!(
                    "  updating root children to {} elements",
                    child_elements.len()
                );
                let root_mut = Arc::make_mut(root);
                root_mut.children = child_elements;
                root_mut.style = crate::element::ElementStyle::default();
            }
        } else {
            log::warn!("  root element not found!");
        }
    }

    /// 更新元素树
    pub fn update_element_tree(&self) {
        let mut tree = self
            .element_tree
            .lock()
            .expect("Failed to acquire element_tree lock in update_element_tree");
        *tree = Some(self.get_root_element());
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    /// 默认窗口状态实例 (目前只支持单个窗口)
    pub static ref WINDOW_STATE: WindowState = WindowState::new();
}
