use crate::ast::minim_model::*;

/// OpenAPI Example Element
#[derive(Debug, Clone)]
pub struct ExampleElement {
    pub object: ObjectElement,
}

impl ExampleElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("example");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("example");
        Self { object: content }
    }

    pub fn summary(&self) -> Option<&StringElement> {
        self.object.get("summary").and_then(Element::as_string)
    }

    pub fn set_summary(&mut self, val: StringElement) {
        self.object.set("summary", Element::String(val));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
    }

    pub fn value(&self) -> Option<&Element> {
        self.object.get("value")
    }

    pub fn set_value(&mut self, val: Element) {
        self.object.set("value", val);
    }

    pub fn external_value(&self) -> Option<&StringElement> {
        self.object.get("externalValue").and_then(Element::as_string)
    }

    pub fn set_external_value(&mut self, val: StringElement) {
        self.object.set("externalValue", Element::String(val));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element){ let k=key.into(); self.object.set(&k, value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }
    pub fn remove(&mut self, key: &str)->Option<Element>{ if let Some(pos)=self.object.content.iter().position(|m|{ if let Element::String(s)=&*m.key { s.content==key } else { false } }){ let m=self.object.content.remove(pos); Some(*m.value) } else { None } }
    pub fn keys(&self)->impl Iterator<Item=&String>{ self.object.content.iter().filter_map(|m|{ if let Element::String(s)=&*m.key { Some(&s.content) } else { None } }) }
    pub fn values(&self)->impl Iterator<Item=&Element>{ self.object.content.iter().map(|m| m.value.as_ref()) }
    pub fn len(&self)->usize{ self.object.content.len() }
    pub fn is_empty(&self)->bool{ self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key:&str)->Option<&Element>{ if key.starts_with("x-"){ self.get_field(key) } else { None } }
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element){ let k=key.into(); if k.starts_with("x-"){ self.set_field(&k, value); } }

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(), String>{
        let has_value=self.value().is_some();
        let has_external=self.external_value().map(|e|!e.content.trim().is_empty()).unwrap_or(false);
        if !has_value && !has_external { return Err("ExampleElement requires either value or externalValue".into()); }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ExampleElement{ fn from(obj:ObjectElement)->Self{ ExampleElement::with_content(obj) }}
impl From<ExampleElement> for ObjectElement{ fn from(el:ExampleElement)->Self{ el.object }}