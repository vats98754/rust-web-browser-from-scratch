use crate::dom::{Node, NodeType, ElementData};
use crate::css::{Stylesheet, Rule, Selector, SimpleSelector, Value, Specificity, Origin};
use std::collections::HashMap;

type PropertyMap = HashMap<String, Value>;

pub struct StyledNode<'a> {
    pub node: &'a Node, // pointer to a DOM tree node
    pub specified_values: PropertyMap, // hashmap (property name, value)
    pub children: Vec<StyledNode<'a>>
}

pub enum Display {
    Inline,
    Block,
    None
}

// Cascade order: (origin_importance, specificity, source_order)
type CascadeKey = (u8, Specificity, usize);

#[derive(Clone)]
struct CascadedDeclaration<'a> {
    declaration: &'a crate::css::Declaration,
    cascade_key: CascadeKey,
}

// Properties that inherit by default
const INHERITED_PROPERTIES: &[&str] = &[
    "color", "font-family", "font-size", "font-style", "font-weight", 
    "line-height", "text-align", "text-decoration", "text-indent",
    "visibility", "white-space", "word-spacing", "letter-spacing"
];

// Initial values for properties
fn get_initial_value(property: &str) -> Value {
    match property {
        "display" => Value::Keyword("inline".to_string()),
        "color" => Value::ColorValue(crate::css::Color { r: 0, g: 0, b: 0, a: 255 }),
        "font-size" => Value::Length(16.0, crate::css::Unit::Px),
        "font-weight" => Value::Keyword("normal".to_string()),
        "font-style" => Value::Keyword("normal".to_string()),
        "text-align" => Value::Keyword("left".to_string()),
        "text-decoration" => Value::Keyword("none".to_string()),
        "visibility" => Value::Keyword("visible".to_string()),
        "white-space" => Value::Keyword("normal".to_string()),
        _ => Value::Keyword("initial".to_string()),
    }
}

fn is_inherited_property(property: &str) -> bool {
    INHERITED_PROPERTIES.contains(&property)
}

impl<'a> StyledNode<'a> {
    // return value of specified property name if it exists
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .or_else(|| self.value(fallback_name))
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

// Enhanced style_tree that supports multiple stylesheets and parent context
pub fn style_tree<'a>(root: &'a Node, stylesheets: &'a [Stylesheet]) -> StyledNode<'a> {
    style_tree_with_parent(root, stylesheets, None)
}

fn style_tree_with_parent<'a>(
    node: &'a Node, 
    stylesheets: &'a [Stylesheet], 
    parent_values: Option<&PropertyMap>
) -> StyledNode<'a> {
    let specified_values = match node.node_type {
        NodeType::Element(ref elem) => {
            let mut values = specified_values(elem, stylesheets);
            apply_inheritance(&mut values, parent_values);
            apply_initial_values(&mut values);
            values
        },
        NodeType::Text(_) => {
            let mut values = HashMap::new();
            apply_inheritance(&mut values, parent_values);
            values
        },
        NodeType::Comment(_) => HashMap::new()
    };

    let children = node.children.iter()
        .map(|child| style_tree_with_parent(child, stylesheets, Some(&specified_values)))
        .collect();

    StyledNode {
        node,
        specified_values,
        children
    }
}

// Apply inheritance rules
fn apply_inheritance(values: &mut PropertyMap, parent_values: Option<&PropertyMap>) {
    if let Some(parent) = parent_values {
        for (property, value) in parent {
            // Inherit if property is explicitly set to inherit OR if it's an inherited property and not set
            if values.get(property) == Some(&Value::Inherit) || 
               (is_inherited_property(property) && !values.contains_key(property)) {
                values.insert(property.clone(), value.clone());
            }
        }
    }
}

// Apply initial values for unset properties
fn apply_initial_values(values: &mut PropertyMap) {
    // For commonly used properties, set initial values if not specified
    let important_properties = ["display", "color", "font-size"];
    for &property in &important_properties {
        if !values.contains_key(property) {
            values.insert(property.to_string(), get_initial_value(property));
        }
    }
}

// Enhanced specified_values function with cascading support
pub fn specified_values(elem: &ElementData, stylesheets: &[Stylesheet]) -> PropertyMap {
    let mut cascaded_declarations: Vec<CascadedDeclaration> = Vec::new();
    
    // Collect declarations from all stylesheets
    for stylesheet in stylesheets {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            if let Some((specificity, _)) = match_rule(elem, rule) {
                for declaration in &rule.declarations {
                    let origin_importance = match (&stylesheet.origin, declaration.important) {
                        (Origin::UserAgent, false) => 0,
                        (Origin::User, false) => 1,
                        (Origin::Author, false) => 2,
                        (Origin::UserAgent, true) => 3,
                        (Origin::User, true) => 4,
                        (Origin::Author, true) => 5,
                    };
                    
                    cascaded_declarations.push(CascadedDeclaration {
                        declaration,
                        cascade_key: (origin_importance, specificity, rule_index),
                    });
                }
            }
        }
    }
    
    // Check for style attribute
    let mut style_declarations = Vec::new();
    if let Some(style_attr) = elem.attrs.get("style") {
        // Parse style attribute as CSS declarations
        if let Some(parsed_declarations) = parse_style_attribute(style_attr) {
            style_declarations = parsed_declarations;
        }
    }
    
    // Add style declarations to cascaded declarations
    for declaration in &style_declarations {
        let origin_importance = if declaration.important { 5 } else { 4 };
        cascaded_declarations.push(CascadedDeclaration {
            declaration,
            cascade_key: (origin_importance, (1, 0, 0), 999999), // High specificity
        });
    }
    
    // Sort by cascade order
    cascaded_declarations.sort_by(|a, b| a.cascade_key.cmp(&b.cascade_key));
    
    // Apply declarations in order, later ones override earlier ones
    let mut values = HashMap::new();
    for cascaded in cascaded_declarations {
        values.insert(
            cascaded.declaration.name.clone(), 
            cascaded.declaration.value.clone()
        );
    }
    
    values
}

// Parse style attribute (simplified - reuses CSS parser)
fn parse_style_attribute(style: &str) -> Option<Vec<crate::css::Declaration>> {
    // Wrap in braces to make it a valid CSS rule body
    let wrapped = format!("dummy {{ {} }}", style);
    
    // Try to parse it - if it fails, return None
    if let Ok(stylesheet) = std::panic::catch_unwind(|| {
        crate::css::parse(wrapped, Origin::Author)
    }) {
        if let Some(rule) = stylesheet.rules.first() {
            return Some(rule.declarations.clone());
        }
    }
    None
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
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // check class selectors
    if selector.class.iter().any(|class| !elem.classes().contains(class.as_str())) {
        return false;
    }

    // we didn't find any non-matching selector components
    return true;
}