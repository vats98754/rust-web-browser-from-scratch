pub mod css;
pub mod dom;
pub mod html;
// pub mod layout;
pub mod style;
// pub mod painting;
// pub mod pdf;

use dom::NodeType;

fn main() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div id="main" class="container">
                    <p style="color: red; font-weight: bold !important;">Hello World!</p>
                    <span class="highlight">Styled text</span>
                    <div class="inherit-demo">This inherits color</div>
                </div>
            </body>
        </html>
    "#.to_string();
    
    // parse HTML into DOM
    let dom = html::Parser::parse(html);
    
    // create multiple stylesheets demonstrating the cascade
    let user_agent_css = css::default_user_agent_stylesheet();
    
    let author_css = css::parse("body { color: blue; font-family: Arial; } .container { background-color: #f0f0f0; display: block; } .highlight { color: green; font-weight: bold; } p { color: black; font-weight: normal; }".to_string(), css::Origin::Author);
    
    // Apply stylesheets with proper cascade order
    let stylesheets = vec![user_agent_css, author_css];
    let styled_tree = style::style_tree(&dom, &stylesheets);
    println!("styled tree created");
    
    // test cascade: check if the head element has display: none from user agent
    if let NodeType::Element(ref elem) = styled_tree.node.node_type {
        println!("Root element: {}", elem.tag_name);
    }
    
    // check if proper styled children
    for child in &styled_tree.children {
        if let NodeType::Element(ref elem) = child.node.node_type {
            println!("Child element: {} with {} style properties", 
                elem.tag_name, child.specified_values.len());
        }
    }
}