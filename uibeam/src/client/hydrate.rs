#![cfg(all(feature = "client", hydrate))]

use ::web_sys::{Node, Event};

pub(crate) enum NodeTraverse {
    FirstChild,
    NextSibling,
}

pub(crate) struct NodePath(&'static str);

impl NodePath {
    pub(crate) fn parse(encoded: &'static str) -> Option<Self> {
        encoded.bytes().all(|b| matches!(b, b'0' | b'1')).then(Self(encoded))
    }
    
    pub(crate) fn encode(self) -> String {
        self.0
    }
    
    pub(crate) fn iter(&self) -> impl Iterator<Item = NodeTraverse> {
        self.0.bytes().map(|b| match b {
            b'0' => NodeTraverse::FirstChild,
            b'1' => NodeTraverse::NextSibling,
            _ => unreachable!("invalid `NodePath`: [{}]", self.0)
        })
    }
    
    pub(crate) fn traverse(&self, root: &Node) -> Option<Node> {
        let mut current = root.clone();
        for t in self.iter() {
            current = match t {
                NodeTraverse::FirstChild => current.first_child(),
                NodeTraverse::NextSibling => current.next_sibling(),
            }?;
        }
        Some(current)
    }
}

pub(crate) enum ReactivePart {
    Text {
        value_fn: Box<dyn Fn() -> String>,
    },
    Attribute {
        name: &'static str,
        value_fn: Box<dyn Fn() -> String>,
    },
    /* TODO:
     * If,
     * For,
     */
}
impl ReactivePart {
    pub(crate) fn apply(&self, node: &Node) {
        match self {
            Self::Text { value_fn } => {
                crate::Effect::new(move || {
                    node.set_text_content(value_fn());
                });
            }
            Self::Attribute { name, value_fn } => {
                crate::Effect::new(move || {
                    node.set_attribute(name, value_fn());
                })
            }
        }
    }
}

pub(crate) struct EventListener {
    event_type: &'static str,
    handler: Box<dyn Fn(Event)>,
}
impl EventListener {
    pub(crate) fn apply_in_root(&self, root: &Node, node: &Node) {
        thread_local! {
            static EVENT_CALLBACKS: std::cell::RefCell<Vec<Box<dyn Fn(Event)>>>;
        }

        let callback_index = EVENT_CALLBACKS.with_borrow_mut(|c| {
            let index = c.len();
            c.push(self.handler);
            index
        });
        node.set_attribute(format!("uibeam-{}-id", callback_index));
        
        let wasm_callback_by_id = ::wasm_bindgen::Closure::<dyn Fn(u32, Event)>::new(|eventid, event| {
            EVENT_CALLBACKS.with_borrow(|c| c[eventid](event))
        });
        
        crate::client::runtime::delegate_event(root, self.event_type, wasm_callback_by_id);
    }
}
