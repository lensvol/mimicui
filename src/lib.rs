mod tools;

use std::collections::VecDeque;

use html_parser::{Dom, Node};
use wasm_bindgen::prelude::*;

use crate::tools::scriptifier::NodeScriptifier;

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

        while let Some((parent_name, node)) = stack.pop_front() {
            source_code.push("".to_string());

            let (name, lines) = node_transformer.scriptify(node);
            lines.iter().for_each(|l| source_code.push(l.into()));

            if let Node::Element(el) = node {
                for (_, subchild) in el.children.iter().enumerate() {
                    stack.push_back((name.clone(), subchild));
                }
            }

            relationships.push((parent_name, name));
        }

        let mut previous_parent: Option<&String> = None;
        for (parent, child) in relationships.iter() {
            if let Some(identifier) = &previous_parent {
                if *identifier != parent {
                    source_code.push("".into());
                }
            } else {
                source_code.push("".to_string());
            }

            previous_parent = Some(parent);
            source_code.push(format!("{parent}.appendChild({child});"));
        }
        source_code.push("".to_string());

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
