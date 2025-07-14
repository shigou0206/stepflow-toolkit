use crate::ast::minim_model::*;

/// OpenAPI Discriminator Mapping Element
/// Specialized element for discriminator mapping which is a Map type rather than generic Object
#[derive(Debug, Clone)]
pub struct DiscriminatorMappingElement {
    pub object: ObjectElement,
}

impl DiscriminatorMappingElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("discriminatorMapping");
        obj.add_class("map");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("discriminatorMapping");
        content.add_class("map");
        Self { object: content }
    }

    /// Get a mapping value by key
    pub fn get_mapping(&self, key: &str) -> Option<&StringElement> {
        self.object.get(key).and_then(Element::as_string)
    }

    /// Set a mapping value
    pub fn set_mapping(&mut self, key: &str, value: StringElement) {
        self.object.set(key, Element::String(value));
    }

    /// Get all mapping keys
    pub fn mapping_keys(&self) -> Vec<String> {
        self.object.content.iter()
            .filter_map(|member| {
                if let Element::String(key_str) = &*member.key {
                    Some(key_str.content.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if mapping has a specific key
    pub fn has_mapping(&self, key: &str) -> bool {
        self.object.has_key(key)
    }

    /// Get the number of mappings
    pub fn mapping_count(&self) -> usize {
        self.object.content.len()
    }

    /// Iterate over all mappings
    pub fn mappings(&self) -> impl Iterator<Item = (&str, &StringElement)> {
        self.object.content.iter().filter_map(|member| {
            if let (Element::String(key), Element::String(value)) = (&*member.key, &*member.value) {
                Some((key.content.as_str(), value))
            } else {
                None
            }
        })
    }
} 