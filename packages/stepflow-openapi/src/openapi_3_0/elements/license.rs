use crate::ast::minim_model::*;

/// OpenAPI `License` Element
#[derive(Debug, Clone)]
pub struct LicenseElement {
    pub object: ObjectElement,
}

impl LicenseElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("license");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("license");
        Self { object: content }
    }

    pub fn name(&self) -> Option<&StringElement> {
        self.object.get("name").and_then(Element::as_string)
    }

    pub fn set_name(&mut self, value: StringElement) {
        self.object.set("name", Element::String(value));
    }

    pub fn url(&self) -> Option<&StringElement> {
        self.object.get("url").and_then(Element::as_string)
    }

    pub fn set_url(&mut self, value: StringElement) {
        self.object.set("url", Element::String(value));
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
    pub fn has_key(&self, key: &str) -> bool {
        self.object.has_key(key)
    }

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
        let name_ok = self.name().map(|n| !n.content.trim().is_empty()).unwrap_or(false);
        if !name_ok { return Err("LicenseElement.name must be non-empty".to_string()); }
        if let Some(url) = self.url() {
            if !url.content.starts_with("http") {
                return Err("LicenseElement.url must start with http/https".to_string());
            }
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for LicenseElement {
    fn from(obj: ObjectElement) -> Self { LicenseElement::with_content(obj) }
}

impl From<LicenseElement> for ObjectElement {
    fn from(el: LicenseElement) -> Self { el.object }
}