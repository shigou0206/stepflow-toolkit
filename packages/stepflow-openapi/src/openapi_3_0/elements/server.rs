use crate::ast::minim_model::*;

/// OpenAPI Server Element
#[derive(Debug, Clone)]
pub struct ServerElement {
    pub object: ObjectElement,
}

impl ServerElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("server");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("server");
        Self { object: content }
    }

    pub fn url(&self) -> Option<&StringElement> {
        self.object.get("url").and_then(Element::as_string)
    }

    pub fn set_url(&mut self, val: StringElement) {
        self.object.set("url", Element::String(val));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
    }

    pub fn variables(&self) -> Option<&ObjectElement> {
        self.object.get("variables").and_then(Element::as_object)
    }

    pub fn set_variables(&mut self, val: ObjectElement) {
        self.object.set("variables", Element::Object(val));
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
        let url_ok = self.url().map(|u| !u.content.trim().is_empty()).unwrap_or(false);
        if !url_ok {
            return Err("ServerElement.url is required".to_string());
        }
        if let Some(url_el) = self.url() {
            if !url_el.content.starts_with("http") && !url_el.content.starts_with("/") {
                return Err("ServerElement.url should start with http/https or be a relative URL".to_string());
            }
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ServerElement {
    fn from(obj: ObjectElement) -> Self { ServerElement::with_content(obj) }
}

impl From<ServerElement> for ObjectElement {
    fn from(el: ServerElement) -> Self { el.object }
}