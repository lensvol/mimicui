use html_parser::{Element, Node};

use crate::tools::sanitizer::Sanitizer;

pub struct NodeScriptifier {
    sanitizer: Sanitizer,
}

impl NodeScriptifier {
    pub fn new() -> NodeScriptifier {
        NodeScriptifier {
            sanitizer: Sanitizer::new(),
        }
    }

    pub fn scriptify_text(&mut self, text: &str) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();
        let sanitized = self.sanitizer.sanitize_name("text");

        result.push(format!(
            "const {sanitized} = document.createTextNode('{}');",
            self.sanitizer.sanitize_text(text)
        ));
        (sanitized, result)
    }

    pub fn scriptify_element(&mut self, element: &Element) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();

        let name = if let Some(node_id) = &element.id {
            node_id
        } else {
            &element.name
        };
        let sanitized = self.sanitizer.sanitize_name(name);
        result.push(format!(
            "const {} = document.createElement('{}');",
            sanitized, &element.name
        ));

        if !element.classes.is_empty() {
            result.push(format!(
                "{}.classList.add('{}');",
                sanitized,
                element.classes.join("', '")
            ));
        }

        for (attribute, optional_value) in &element.attributes {
            let value = optional_value.clone().unwrap_or("".to_string());
            if attribute == "style" {
                result.push(format!("{sanitized}.style.cssText = '{value}';"));
            } else if attribute.starts_with("data-") {
                result.push(format!("{sanitized}.dataset.{attribute} = '{value}';",));
            } else {
                result.push(format!(
                    "{sanitized}.setAttribute('{attribute}', '{value}');",
                ));
            }
        }

        (sanitized, result)
    }

    pub fn scriptify_comment(&mut self, comment: &str) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();
        let name = self.sanitizer.sanitize_name("comment");
        let sanitized_comment = self.sanitizer.sanitize_text(comment);

        result.push(format!(
            "const {name} = document.createComment('{sanitized_comment}');",
        ));
        (name, result)
    }

    pub fn scriptify(&mut self, node: &Node) -> (String, Vec<String>) {
        match node {
            Node::Text(text) => self.scriptify_text(text),
            Node::Element(element) => self.scriptify_element(element),
            Node::Comment(text) => self.scriptify_comment(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::scriptifier::NodeScriptifier;
    use html_parser::Node;

    #[test]
    fn test_simple_text_is_scriptified_correctly() {
        let mut scriptifier = NodeScriptifier::new();
        let (name, code) = scriptifier.scriptify(&Node::Text("Hello, world!".into()));

        assert_eq!(name, "text");
        assert_eq!(
            code,
            vec!["const text = document.createTextNode('Hello, world!');"]
        );
    }

    #[test]
    fn test_special_symbols_in_comment_are_handled_properly() {
        let comment = "Hey\\n".to_string();
        let mut scriptifier = NodeScriptifier::new();
        let (name, code) = scriptifier.scriptify(&Node::Comment(comment));

        assert_eq!(name, "comment");
        assert_eq!(
            code,
            vec!["const comment = document.createComment('Hey\\\\n');"]
        );
    }
}
