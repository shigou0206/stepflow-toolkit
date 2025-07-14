use crate::ast::minim_model::*;
use crate::openapi_3_0::elements::path_item::PathItemElement;
/// OpenAPI Paths Element
#[derive(Debug, Clone)]
pub struct PathsElement {
    pub object: ObjectElement,
}

impl PathsElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("paths");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("paths");
        Self { object: content }
    }

    // 示例接口，可按需扩展
    pub fn get_path(&self, path: &str) -> Option<&Element> {
        self.object.get(path)
    }

    pub fn set_path(&mut self, path: &str, value: Element) {
        self.object.set(path, value);
    }

    // -------- Generic accessors --------
    pub fn get(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into(); self.object.set(&k, value); }

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
    /// 至少包含一个以 `/` 开头的 path，且每个 pathItem 非空
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.is_empty() { return Err("PathsElement must contain at least one path".to_string()); }
        for key in self.keys() {
            if !key.starts_with('/') {
                return Err(format!("Path '{}' must start with '/'", key));
            }
            if let Some(Element::Object(obj)) = self.get(key) {
                if obj.content.is_empty() {
                    return Err(format!("PathItem '{}' must not be empty", key));
                }
            } else {
                return Err(format!("Path '{}' must be an object", key));
            }
        }
        Ok(())
    }

    // -------- Derived helpers --------
    /// 将所有符合条件的 pathItem 转换为结构化 PathItemElement
    pub fn path_items(&self) -> Vec<(&str, PathItemElement)> {
        self.keys().filter_map(|k| {
            self.get(k).and_then(Element::as_object).map(|obj| (k.as_str(), PathItemElement::with_content(obj.clone())))
        }).collect()
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for PathsElement {
    fn from(obj: ObjectElement) -> Self { PathsElement::with_content(obj) }
}

impl From<PathsElement> for ObjectElement {
    fn from(el: PathsElement) -> Self { el.object }
}