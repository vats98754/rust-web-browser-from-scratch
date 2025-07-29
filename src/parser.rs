struct Parser {
    pos: usize,
    input: String
}

impl Parser {
    // Read the current character without consuming it
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Do the coming chars start with the provided string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string `s` is found at the current position, consume it; otherwise, panic
    fn expect(&mut self, s: &str) {
        if self.starts_with(s) {
            self.pos += s.len();
        } else {
            panic!("Expected {:?} at byte {} but it was not found", s, self.pos);
        }
    }

    // Return true if all input is consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        return c;
    }

    // Consume characters until `test` returns false
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char()); // push the consumed char to the result String
        }
        return result;
    }

    // Consume and discard any number of whitespaces
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace)
    }

    // Parse a tag or attribute name
    fn parse_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    // Parse a single node
    fn parse_node(&mut self) -> dom::Node {
        if self.starts_with("<!") {
            self.parse_comment();
        } else if self.starts_with("<") {
            self.parse_element();
        } else {
            self.parse_text();
        }
    }

    // In our subset of HTML, text node can contain any char except "<"
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }
    
    // In our subset of HTML, comment node can contain any char except -
    fn parse_comment(&mut self) -> dom::Node {
        self.expect("<!--");
        let text = self.consume_while(|c| c != '-');
        self.expect("-->");

        return dom::comment(text);
    }

    // Element node contains open and close tag
    fn parse_element(&mut self) -> dom::Node {
        // Opening tag
        self.expect("<");
        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();
        self.expect(">");

        // Contents
        let children = self.parse_nodes();

        // Closing tag
        self.expect("</");
        self.expect(tag_name);
        self.expect(">");

        return dom::elem(tag_name, attrs, children);
    }

    // parse a single name="value" pair
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_name(); // attribute name
        self.expect("=");
        let value = self.parse_attr_value(); // attribute value
        return (name, value);
    }

    // parse a quoted value
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = consume_while(|c| c != open_quote)
        let close_quote = self.consume_char();
        assert_eq(open_quote, close_quote);
        return value;
    }

    // parse a list of name="value" pairs, separated by whitespace
    fn parse_attributes(&mut self) -> dom::AttrsMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        return attributes;
    }

    // parse a sequence of sibling nodes
    fn parse_nodes(&mut self) {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        return nodes;
    }

    // parse entire HTML doc and return its root element
    pub fn parse(source: String) -> dom::Node {
        let mut nodes = Parser { pos = 0, input: source }.parse_nodes();
        // if the DOM contains a root element, return it; otherwise, create one
        if nodes.len() == 1 {
            return nodes.remove(0);
        } else {
            return dom::elem("html".to_string(), HashMap::new(), nodes);
        }
    }
}