use crate::ast::minim_model::*;

#[derive(Debug, Clone)]
pub struct PathItemElement {
    pub object: ObjectElement,
}

impl PathItemElement {
    pub fn new() -> Self {
        let mut object = ObjectElement::new();
        object.set_element_type("pathItem");
        Self { object }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("pathItem");
        Self { object: content }
    }

    pub fn ref_(&self) -> Option<&StringElement> {
        self.object.get("$ref").and_then(Element::as_string)
    }

    pub fn set_ref(&mut self, val: StringElement) {
        self.object.set("$ref", Element::String(val));
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

    pub fn operation(&self, method: &str) -> Option<&Element> {
        self.object.get(method)
    }

    pub fn set_operation(&mut self, method: &str, op: Element) {
        self.object.set(method, op);
    }

    pub fn servers(&self) -> Option<&ArrayElement> {
        self.object.get("servers").and_then(Element::as_array)
    }

    pub fn set_servers(&mut self, val: ArrayElement) {
        self.object.set("servers", Element::Array(val));
    }

    pub fn parameters(&self) -> Option<&ArrayElement> {
        self.object.get("parameters").and_then(Element::as_array)
    }

    pub fn set_parameters(&mut self, val: ArrayElement) {
        self.object.set("parameters", Element::Array(val));
    }

    // 快捷方法：HTTP 操作
    pub fn get(&self) -> Option<&Element> {
        self.operation("get")
    }

    pub fn post(&self) -> Option<&Element> {
        self.operation("post")
    }

    pub fn put(&self) -> Option<&Element> {
        self.operation("put")
    }

    pub fn delete(&self) -> Option<&Element> {
        self.operation("delete")
    }

    pub fn patch(&self) -> Option<&Element> {
        self.operation("patch")
    }

    pub fn head(&self) -> Option<&Element> {
        self.operation("head")
    }

    pub fn options(&self) -> Option<&Element> {
        self.operation("options")
    }

    pub fn trace(&self) -> Option<&Element> {
        self.operation("trace")
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn set_field(&mut self, key: impl Into<String>, value: Element) {
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
        if key.starts_with("x-") { self.get_field(key) } else { None }
    }

    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into(); if k.starts_with("x-") { self.set_field(&k, value); }
    }

    // -------- Basic validation --------
    /// 必须包含至少一个操作或 $ref
    pub fn validate_basic(&self) -> Result<(), String> {
        let op_keys = ["get", "put", "post", "delete", "options", "head", "patch", "trace"];
        let has_operation = op_keys.iter().any(|k| self.has_key(k));
        let has_ref = self.ref_().is_some();
        if has_operation || has_ref {
            Ok(())
        } else {
            Err("PathItemElement must contain at least one operation or $ref".to_string())
        }
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for PathItemElement {
    fn from(obj: ObjectElement) -> Self { PathItemElement::with_content(obj) }
}

impl From<PathItemElement> for ObjectElement {
    fn from(el: PathItemElement) -> Self { el.object }
}