use crate::dom;

// a simple selector can include a tag name, an ID prefixed by '#', any number of class
// names prefixed by '.', or some combination of the above. If the tag name is empty
// or '*' then it is a “universal selector” that can match any tag.

// a selector is either a simple selector or a chain of selectors with delimiter: ' ', '+', '>'
struct Stylesheet {
    rules: Vec<Rule>
}

struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>
}

// ways to select an element, could be by its tag_name, id, or list of classes
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>
}

// types of selector, for now just the atomic simple selector is implemented
enum Selector {
    Simple(SimpleSelector)
}

// paired with a selector to specify what properties of selected DOM nodes to apply
struct Declaration {
    name: String, // name of property
    value: Value // value set to this property
}

enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values as required
}

enum Unit {
    Px,
    // insert more units as required
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    // decides which style overrides another if conflict
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        return (a, b, c);
    }
}

pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser { pos: 0, input: source };
    return Stylesheet { rules: parser.parse_rules() };
}

struct Parser {
    pos: usize,
    input: String
}

impl Parser {
    // return true if all chars in input consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn expect_char(&mut self, c: char) {
        if self.consume_char() != c {
            panic!("Expected {:?} at byte {} but it was not found", c, self.pos);
        }
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        return c;
    }

    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        return result;
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // parse an identifier based on valid chars for an identifier name
    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..'9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier())
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        self.consume_while(|c| matches!(c, '0'..'9' | '.')).parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            other => panic!("unit '{}' not recognized", other)
        }
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos .. self.pos+2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_color(&mut self) -> Value {
        self.expect_char('#');
        Value::ColorValue(
            Color {
                r: self.parse_hex_pair(),
                g: self.parse_hex_pair(),
                b: self.parse_hex_pair(),
                a: 255
            }
        )
    }

    // parse a simple selector `type#id.class1.class2.class3`
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector { tag_name: None, id: None, class: Vec::new() };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        return selector;
    }

    // parse a comma-separated list of selectors
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => { self.consume_char(); self.consume_whitespace(); }
                '{' => break, // start of declarations
                c => panic!("Unexpected char {} in selector list", c)
            }
        }
        // return selectors with highest specificity first, used in matching
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        return selectors;
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        self.expect_char(':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        self.expect_char(';');
        return Declaration { name, value }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        self.expect_char('{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        return declarations;
    }

    // parse a rule set: `<selectors> { <declarations> }`
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations()
        }
    }

    // parse a list of rules to create a stylesheet
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() { break }
            rules.push(self.parse_rule());
        }
        return rules;
    }
}

// test if current char matches the allowed chars
fn valid_identifier_char(c: char) -> bool {
    // TODO: Include U+00A0 and higher.
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}