use html_parser::{Dom, Element, Node};
use std::collections::{HashMap, VecDeque};
use std::io::{stdin, Read};
use std::{env, fs, str};

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

    fn sanitize_name(&mut self, base: &String) -> String {
        let current_mark = self.registry.entry(base.clone()).or_insert(0);
        *current_mark += 1;

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

        return if *current_mark == 1 {
            normalized
        } else {
            format!("{}{}", normalized, *current_mark)
        };
    }
}

impl NodeScriptifier {
    fn new() -> NodeScriptifier {
        NodeScriptifier {
            sanitizer: NameSanitizer::new(),
        }
    }

    fn scriptify(&mut self, node: &Node) -> (String, Vec<String>) {
        let mut result: Vec<String> = Vec::new();

        if let Node::Text(text) = node {
            let name = self.sanitizer.sanitize_name(&("text".to_string()));
            result.push(format!(
                "const {} = document.createTextNode('{}');",
                name, text
            ));
            return (name, result);
        }

        if let Node::Element(element) = node {
            let name = if let Some(node_id) = &element.id {
                node_id
            } else {
                &element.name
            };
            let sanitized = self.sanitizer.sanitize_name(&name);
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
                // TODO: Maybe handle null values?
                if let Some(value) = optional_value {
                    if attribute == "style" {
                        result.push(format!("{}.style.cssText = '{}';", sanitized, value));
                    } else if attribute.starts_with("data-") {
                        result.push(format!(
                            "{}.dataset.{} = '{}';",
                            sanitized, attribute, value
                        ));
                    } else {
                        result.push(format!(
                            "{}.setAttribute('{}', '{}');",
                            sanitized, attribute, value
                        ));
                    }
                }
            }

            return (sanitized, result);
        }

        unreachable!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut scriptifier = NodeScriptifier::new();
    let mut relationships: Vec<(String, String)> = Vec::new();
    let mut root_element: Option<String> = None;
    let mut source_code: Vec<String> = Vec::new();
    let mut buffer = Vec::new();

    if args.len() < 2 {
        println!("Usage: mimicui < - | FILE >");
        std::process::exit(-1);
    }

    let source = args.get(1).unwrap();

    if source == "-" {
        stdin()
            .read_to_end(&mut buffer)
            .expect("Nothing to process!");
    } else {
        buffer = Vec::from(fs::read_to_string("fragment.html").unwrap());
    }

    let dom = Dom::parse(str::from_utf8(&buffer).unwrap()).unwrap();

    let mut stack: VecDeque<(usize, Option<String>, &Node)> =
        dom.children.iter().map(|n| (0, None, n)).collect();

    if stack.len() > 1 {
        source_code.push("".to_string());
        source_code.push("const root = document.createElement('div');".to_string());
        source_code.push("".to_string());

        root_element = Some("root".to_string());
    }

    while let Some((indent, parent_name, node)) = stack.pop_front() {
        if let Node::Comment(_) = node {
            continue;
        }

        let (name, lines) = scriptifier.scriptify(node);
        if root_element.is_none() {
            root_element = Some(name.clone());
        }

        lines.iter().for_each(|l| source_code.push(l.into()));

        if let Node::Element(el) = node {
            for (_, subchild) in el.children.iter().enumerate() {
                stack.push_back((indent + 4, Some(name.clone()), subchild));
            }
        }

        source_code.push("".to_string());
        if let Some(node_name) = parent_name {
            relationships.push((node_name, name));
        }
    }

    for (parent, child) in relationships.iter() {
        source_code.push(format!("{}.appendChild({});", parent, child));
    }
    source_code.push(format!("return {};", root_element.unwrap()));

    println!("function createMimic() {{");
    source_code.iter().for_each(|l| println!("    {}", l));
    println!("}}");

    std::process::exit(0);
}
