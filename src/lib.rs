use html_parser::{Dom, Element, Node};
use std::collections::{HashMap, VecDeque};

use wasm_bindgen::prelude::*;

struct NameSanitizer {
    registry: HashMap<String, u8>,
}

struct NodeScriptifier {
    sanitizer: NameSanitizer,
}

impl NameSanitizer {
    fn new() -> NameSanitizer {
        NameSanitizer {
            registry: Default::default(),
        }
    }

    fn sanitize_name(&mut self, base: &str) -> String {
        let name = match base {
            "a" => "link".to_string(),
            "div" => "container".to_string(),
            "b" => "bold".to_string(),
            "i" => "italics".to_string(),
            "pre" => "preformatted".to_string(),
            "code" => "codeBlock".to_string(),
            "p" => "paragraph".to_string(),
            "h" => "heading".to_string(),
            "span" => "textSpan".to_string(),
            _ => {
                let parts = base
                    .split(&['_', '-'])
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<&str>>();

                let mut normalized = String::with_capacity(base.len());
                normalized.push_str(parts.first().unwrap());

                for part in parts.iter().skip(1) {
                    normalized.push(
                        part.chars()
                            .into_iter()
                            .next()
                            .unwrap()
                            .to_ascii_uppercase(),
                    );
                    normalized.push_str(&part[1..]);
                }
                normalized
            }
        };

        let current_mark = self.registry.entry(name.clone()).or_insert(0);
        *current_mark += 1;

        return if *current_mark == 1 {
            name
        } else {
            format!("{}{}", name, *current_mark)
        };
    }
}

impl NodeScriptifier {
    fn new() -> NodeScriptifier {
        NodeScriptifier {
            sanitizer: NameSanitizer::new(),
        }
    }

    fn scriptify_text(&mut self, text: &str) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();
        let sanitized = self.sanitizer.sanitize_name("text");

        result.push(format!(
            "const {sanitized} = document.createTextNode('{text}');",
        ));
        (sanitized, result)
    }

    fn scriptify_element(&mut self, element: &Element) -> (String, Vec<String>) {
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

    fn scriptify_comment(&mut self, comment: &str) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();
        let name = self.sanitizer.sanitize_name("comment");

        result.push(format!(
            "const {name} = document.createComment('{comment}');",
        ));
        (name, result)
    }

    fn scriptify(&mut self, node: &Node) -> (String, Vec<String>) {
        match node {
            Node::Text(text) => self.scriptify_text(text),
            Node::Element(element) => self.scriptify_element(element),
            Node::Comment(text) => self.scriptify_comment(text),
        }
    }
}

pub struct HTMLScriptifier {}

impl Default for HTMLScriptifier {
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLScriptifier {
    pub fn new() -> Self {
        HTMLScriptifier {}
    }

    pub fn scriptify_html(&mut self, source: &str) -> String {
        let mut node_transformer = NodeScriptifier::new();
        let mut relationships: Vec<(String, String)> = Vec::new();
        let mut source_code: Vec<String> = Vec::new();

        let dom = Dom::parse(source).unwrap();

        let mut stack: VecDeque<(String, &Node)> = dom
            .children
            .iter()
            .map(|n| ("root".to_string(), n))
            .collect();

        source_code.push("".to_string());
        source_code.push("const root = document.createElement('div');".to_string());
        source_code.push("".to_string());

        while let Some((parent_name, node)) = stack.pop_front() {
            let (name, lines) = node_transformer.scriptify(node);
            lines.iter().for_each(|l| source_code.push(l.into()));

            if let Node::Element(el) = node {
                for (_, subchild) in el.children.iter().enumerate() {
                    stack.push_back((name.clone(), subchild));
                }
            }

            source_code.push("".to_string());
            relationships.push((parent_name, name));
        }

        let mut previous_parent: Option<&String> = None;
        for (parent, child) in relationships.iter() {
            if let Some(identifiter) = &previous_parent {
                if *identifiter != parent {
                    source_code.push("".into());
                }
            }

            previous_parent = Some(parent);
            source_code.push(format!("{parent}.appendChild({child});"));
        }
        source_code.push("".into());
        source_code.push("return root;".into());

        let mut result = String::new();
        result.push_str("function createMimic() {");
        source_code.iter().for_each(|l| {
            result.push_str("    ");
            result.push_str(l);
            result.push('\n');
        });
        result.push('}');

        result
    }
}

#[wasm_bindgen]
pub fn html_to_js(html_code: String) -> String {
    HTMLScriptifier::new().scriptify_html(&html_code)
}
