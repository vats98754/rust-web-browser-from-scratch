use crate::dom::{Node, NodeType, ElementData};
use crate::css::{Stylesheet, Rule, Selector, SimpleSelector, Value, Specificity};
use std::collections::HashMap;

type PropertyMap = HashMap<String, Value>;

struct StyledNode<'a> {
    node: &'a Node, // pointer to a DOM tree node
    specified_values: PropertyMap, // hashmap (property name, value)
    children: Vec<StyledNode<'a>>
}

pub enum Display {
    Inline,
    Block,
    None
}

impl<'a> StyledNode<'a> {
    // return value of specified property name if it exists
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name).unwrap_or_else(|| self.value(fallback_name))
                        .unwrap_or_else(|| default.clone())
    }

    // return value of display property with default inline
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline
            },
            _ => Display::Inline
        }
    }
}

// apply a stylesheet to an entire DOM tree, returns a StyledNode tree root
pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
            NodeType::Comment(_) => HashMap::new()
        },
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect()
    }
}

// apply a selector to a single element, returning the PropertyMap of that element's styles
pub fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // go through rules from low to high specificity
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    return values;
}

type MatchedNode<'a> = (Specificity, &'a Rule);

// go through rules in stylesheet and filter which rules match the element
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedNode<'a>> {
    // linear scan of rules for now; for larger DOM trees, store rules in Hashmap based on tag_name, id, class
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

// if the element matches the rule, return a MatchedNode (specificity of selector, rule)
fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedNode<'a>> {
    rule.selectors.iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

// if the element matches the selector, return true
fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(s) => matches_simple_selector(elem, s)
    }
}

// if the elem name, id, or classes match selector, return true
fn matches_simple_selector(elem: &ElementData, selector: &Selector) -> bool {
    // check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // check class selectors
    if selector.classes.iter().any(|class| !elem.classes().contains(class.as_str())) {
        return false;
    }

    // we didn't find any non-matching selector components
    return true;
}