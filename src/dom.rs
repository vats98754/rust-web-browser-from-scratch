use std::collections::{HashMap, HashSet};

// Data struct and type defns

pub struct CommentData {
    pub comment: String
}

pub struct ElementData {
    pub tag_name: String,
    pub attrs: AttrsMap
}

pub type AttrsMap = HashMap<String, String>;

pub enum NodeType {
    Text(String),
    Element(ElementData),
    Comment(CommentData)
}

pub struct Node {
    // data common to all Node structs
    pub children: Vec<Node>,
    // data specific to each node type
    pub node_type: NodeType
}

// Constructor definitions

pub fn text(data: String) -> Node {
    Node {
        children: vec![],
        node_type: NodeType::Text(data)
    }
}

pub fn elem(tag_name: String, attrs: AttrsMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData { tag_name, attrs })
    }
}

pub fn comment(comment: String) -> Node {
    Node {
        children: vec![],
        node_type: NodeType::Comment(CommentData { comment })
    }
}

// Element methods

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attrs.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attrs.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new()
        }
    }
}
