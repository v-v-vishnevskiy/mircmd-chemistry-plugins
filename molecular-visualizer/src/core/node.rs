use std::sync::atomic::{AtomicU32, Ordering};

use super::Transform;

const MAX_ID: u32 = 255 * 255 * 255;

static ID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn next_id() -> u32 {
    let id = ID_COUNTER.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
        Some(if current >= MAX_ID { 0 } else { current + 1 })
    });

    id.unwrap_or(0)
}

pub struct Node {
    pub picking_id: u32,
    pub transform: Transform,
    pub color: [f32; 4],
    pub children: Vec<Node>,
    pub container: bool,
    pub visible: bool,
}

impl Node {
    pub fn new() -> Self {
        Self {
            picking_id: next_id(),
            transform: Transform::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            children: Vec::new(),
            container: false,
            visible: true
        }
    }
}
