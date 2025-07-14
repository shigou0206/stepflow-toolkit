use crate::ast::minim_model::*;

/// OpenAPI ServerVariable Element
#[derive(Debug, Clone)]
pub struct ServerVariableElement {
    pub object: ObjectElement,
}

impl ServerVariableElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("serverVariable");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("serverVariable");
        Self { object: content }
    }

    pub fn enum_values(&self) -> Option<&ArrayElement> {
        self.object.get("enum").and_then(Element::as_array)
    }

    pub fn set_enum_values(&mut self, value: ArrayElement) {
        self.object.set("enum", Element::Array(value));
    }

    pub fn default_value(&self) -> Option<&StringElement> {
        self.object.get("default").and_then(Element::as_string)
    }

    pub fn set_default_value(&mut self, value: StringElement) {
        self.object.set("default", Element::String(value));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, value: StringElement) {
        self.object.set("description", Element::String(value));
    }

    // -------- Generic accessors --------
    pub fn get(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        self.object.set(&k, value);
    }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }

    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|member| {
            if let Element::String(s) = &*member.key { s.content == key } else { false }
        }) {
            let member = self.object.content.remove(pos);
            Some(*member.value)
        } else { None }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.object.content.iter().filter_map(|m| {
            if let Element::String(s) = &*m.key { Some(&s.content) } else { None }
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &Element> {
        self.object.content.iter().map(|m| m.value.as_ref())
    }

    pub fn len(&self) -> usize { self.object.content.len() }
    pub fn is_empty(&self) -> bool { self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key: &str) -> Option<&Element> {
        if key.starts_with("x-") { self.get(key) } else { None }
    }

    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into(); if k.starts_with("x-") { self.set(&k, value); }
    }

    // -------- Basic validation --------
    pub fn validate_basic(&self) -> Result<(), String> {
        // default 必须存在且非空
        let default_ok = self.default_value().map(|d| !d.content.trim().is_empty()).unwrap_or(false);
        if !default_ok {
            return Err("ServerVariableElement.default is required".to_string());
        }
        // 如果有 enum，则 default 必须包含在 enum 中
        if let (Some(enum_arr), Some(default_val)) = (self.enum_values(), self.default_value()) {
            let found = enum_arr.content.iter().any(|el| {
                if let Element::String(s) = el { s.content == default_val.content } else { false }
            });
            if !found {
                return Err("ServerVariableElement.default must be one of enum values".to_string());
            }
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ServerVariableElement {
    fn from(obj: ObjectElement) -> Self { ServerVariableElement::with_content(obj) }
}

impl From<ServerVariableElement> for ObjectElement {
    fn from(el: ServerVariableElement) -> Self { el.object }
}