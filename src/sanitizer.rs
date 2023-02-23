use std::collections::HashMap;

pub struct Sanitizer {
    name_registry: HashMap<String, u8>,
}

impl Sanitizer {
    pub fn new() -> Sanitizer {
        Sanitizer {
            name_registry: Default::default(),
        }
    }

    pub fn sanitize_name(&mut self, base: &str) -> String {
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

        let current_mark = self.name_registry.entry(name.clone()).or_insert(0);
        *current_mark += 1;

        return if *current_mark == 1 {
            name
        } else {
            format!("{}{}", name, *current_mark)
        };
    }

    pub fn sanitize_text(&self, base: &str) -> String {
        str::replace(base, "\\", "\\\\")
            .replace('\"', "\\\"")
            .replace('\'', "\\'")
    }
}

#[cfg(test)]
mod tests {
    use crate::sanitizer::Sanitizer;

    #[test]
    fn test_not_seen_before_name_not_changed() {
        let mut sanitizer = Sanitizer::new();
        assert_eq!(sanitizer.sanitize_name("test"), "test");
    }

    #[test]
    fn test_previously_seen_name_changed_to_prevent_collisions() {
        let mut sanitizer = Sanitizer::new();

        let first = sanitizer.sanitize_name("test");
        let second = sanitizer.sanitize_name("test");
        let third = sanitizer.sanitize_name("test");

        assert_eq!(first, "test");
        assert_eq!(second, "test2");
        assert_eq!(third, "test3");
    }

    #[test]
    fn test_seen_special_name_also_gets_modified_to_prevent_collisions() {
        let mut sanitizer = Sanitizer::new();

        let first = sanitizer.sanitize_name("p");
        let second = sanitizer.sanitize_name("p");

        assert_eq!(first, "paragraph");
        assert_eq!(second, "paragraph2");
    }
}
