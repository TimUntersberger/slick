use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Display};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, Element, HtmlElement};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug)]
enum NodeAttributeValue {
    String(String),
    Number(i32),
    Boolean(bool),
}

impl NodeAttributeValue {
    pub fn as_text(&self) -> String {
        match self {
            NodeAttributeValue::String(x) => x.clone(),
            NodeAttributeValue::Number(x) => x.to_string(),
            NodeAttributeValue::Boolean(x) => x.to_string(),
        }
    }
}

impl Display for NodeAttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeAttributeValue::String(x) => format!("\"{}\"", x),
                NodeAttributeValue::Number(x) => x.to_string(),
                NodeAttributeValue::Boolean(x) => x.to_string(),
            }
        )
    }
}

impl From<String> for NodeAttributeValue {
    fn from(x: String) -> Self {
        NodeAttributeValue::String(x)
    }
}

impl<'a> From<&'a str> for NodeAttributeValue {
    fn from(x: &'a str) -> Self {
        NodeAttributeValue::String(x.to_string())
    }
}

impl From<i32> for NodeAttributeValue {
    fn from(x: i32) -> Self {
        NodeAttributeValue::Number(x)
    }
}

impl From<u32> for NodeAttributeValue {
    fn from(x: u32) -> Self {
        NodeAttributeValue::Number(x as i32)
    }
}

trait AnyMessage: std::any::Any + std::fmt::Debug {}

impl<T: std::any::Any + std::fmt::Debug> AnyMessage for T {}

enum NodeKind {
    Native { tag: String },
    Text(String),
    Custom {
        component: Box<dyn Component<Message = ()>>,
        rendered: Box<Node>
    }
}

impl std::fmt::Debug for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Native { tag } => f.debug_struct("Native")
                .field("tag", tag)
                .finish(),
            NodeKind::Text(x) => f.debug_tuple("Text").field(x).finish(),
            NodeKind::Custom { component, rendered } => f.debug_struct("Custom")
                .field("rendered", rendered)
                .finish(),
        }
    }
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
    children: Vec<Node>,
    on_click: Option<Box<dyn AnyMessage>>,
    attributes: HashMap<&'static str, NodeAttributeValue>,
}

/// Constructor helpers
impl Node {
    pub fn native(tag: impl Into<String>) -> Self {
        Self {
            kind: NodeKind::Native { tag: tag.into() },
            children: vec![],
            on_click: None,
            attributes: HashMap::new(),
        }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self {
            kind: NodeKind::Text(value.into()),
            children: vec![],
            on_click: None,
            attributes: HashMap::new(),
        }
    }

    pub fn custom(value: Box<dyn Component<Message = ()>>) -> Self {
        Self {
            kind: NodeKind::Custom {
                rendered: Box::new(value.view()),
                component: value
            },
            children: vec![],
            on_click: None,
            attributes: HashMap::new(),
        }
    }
}

/// Builder methods
impl Node {
    pub fn with_child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }
}

impl Node {
    pub fn to_html(&self) -> String {
        match &self.kind {
            NodeKind::Text(value) => value.clone(),
            NodeKind::Custom { rendered, .. } => rendered.to_html(),
            NodeKind::Native { tag } => format!(
                "<{}{}{}>\n{}\n</{}>",
                tag,
                if self.attributes.len() == 0 { "" } else { " " },
                self.attributes
                    .iter()
                    .map(|(key, val)| format!("{}={}", key, val))
                    .reduce(|acc, x| { format!("{} {}", acc, x) })
                    .unwrap_or_default(),
                self.children
                    .iter()
                    .map(|child| child.to_html())
                    .map(|x| x
                        .split('\n')
                        .map(|line| format!("  {}", line))
                        .reduce(|acc, x| format!("{}\n{}", acc, x))
                        .unwrap_or_default())
                    .reduce(|acc, x| { format!("{}\n{}", acc, x) })
                    .unwrap_or_default(),
                tag
            ),
        }
    }
}

enum Effect {}

trait Component {
    type Message;

    fn view<'a>(&self) -> Node;
    fn update(&mut self, msg: Self::Message) -> Option<Effect>;
}

struct Test;

impl Component for Test {
    type Message = ();

    fn view<'a>(&self) -> Node {
        Node::text("Hello world")
    }

    fn update(&mut self, msg: Self::Message) -> Option<Effect> {
        None
    }
}

struct App;

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

impl Component for App {
    type Message = ();

    fn view<'a>(&self) -> Node {
        Node::native("div").with_child(Node::custom(Box::new(Test {})))
    }

    fn update(&mut self, msg: Self::Message) -> Option<Effect> {
        None
    }
}

// fn render_node(document: Document, target: &str, node: &Node) {
//     let parent = document.query_selector(target).unwrap();
//
//     if let Some(parent) = &parent {
//         parent.append_child(&node_to_element(node, &document)).unwrap();
//     }
// }
//
// fn node_to_element(node: &Node, document: &Document) -> Element {
//     let element = document.create_element(&node.tag).unwrap();
//
//     if let Some(msg) = &node.on_click {
//         // let msg = msg.downcast::<AppMessage>();
//         // let cb = Closure::wrap(Box::new(move || {
//         //     println!("{:?}", msg);
//         // }) as Box<dyn FnMut()>);
//         // element.dyn_ref::<HtmlElement>().unwrap().set_onclick(Some(cb.as_ref().unchecked_ref()));
//     }
//
//     for (key, val) in &node.attributes {
//         element.set_attribute(key, &val.as_text()).unwrap();
//     }
//
//     for child in &node.children {
//         if child.tag == "text" {
//             element.append_child(
//                 &document.create_text_node(&child.attributes.get("value").unwrap().as_text()),
//             ).unwrap();
//         } else {
//             element.append_child(&node_to_element(child, document)).unwrap();
//         }
//     }
//
//     element
// }
//
// thread_local! {
//     static VIRT_DOM: RefCell<Node> = RefCell::default();
// }

pub fn main() {
    // let window = web_sys::window().expect("no global `window` exists");
    // let document = window.document().expect("should have a document on window");

    let virt_dom = App::default().view();

    println!("{:#?}\n\n{}", virt_dom, virt_dom.to_html());

    // render_node(document, "body", &virt_dom.borrow());
}
