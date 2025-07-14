use crate::ast::minim_model::*;

/// OpenAPI Responses Element
#[derive(Debug, Clone)]
pub struct ResponsesElement {
    pub object: ObjectElement,
}

impl ResponsesElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("responses");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("responses");
        Self { object: content }
    }

    /// `default` 字段（可能为 ResponseElement 或 ReferenceElement）
    pub fn default(&self) -> Option<&Element> {
        self.object.get("default")
    }

    pub fn set_default(&mut self, value: Element) {
        self.object.set("default", value);
    }

    // 可扩展方法，如 get("200")、get("404") 等
    pub fn get_status_response(&self, status: &str) -> Option<&Element> {
        self.object.get(status)
    }

    pub fn set_status_response(&mut self, status: &str, value: Element) {
        self.object.set(status, value);
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element){ let k = key.into(); self.object.set(&k, value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }
    pub fn remove(&mut self, key:&str)->Option<Element>{ if let Some(pos)=self.object.content.iter().position(|m|{ if let Element::String(s)=&*m.key { s.content==key } else { false } }){ let m=self.object.content.remove(pos); Some(*m.value) } else { None } }
    pub fn keys(&self)->impl Iterator<Item=&String>{ self.object.content.iter().filter_map(|m|{ if let Element::String(s)=&*m.key { Some(&s.content) } else { None } }) }
    pub fn values(&self)->impl Iterator<Item=&Element>{ self.object.content.iter().map(|m| m.value.as_ref()) }
    pub fn len(&self)->usize{ self.object.content.len() }
    pub fn is_empty(&self)->bool{ self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key:&str)->Option<&Element>{ if key.starts_with("x-"){ self.get_field(key) } else { None } }
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element){ let k=key.into(); if k.starts_with("x-"){ self.set_field(&k, value); } }

    // -------- Basic validation --------
    /// 必须包含 default 或至少一个有效的 HTTP 状态码键
    pub fn validate_basic(&self)->Result<(), String>{
        if self.default().is_none() && self.object.content.iter().all(|m|{
            if let Element::String(s)=&*m.key { !is_status_key(&s.content) } else { true }
        }){
            return Err("ResponsesElement must contain at least one response or default".into());
        }
        Ok(())
    }
}

// 简单检测 HTTP 状态码或模式 (200 / 2XX)
fn is_status_key(key: &str) -> bool {
    if key.len()==3 { key.chars().all(|c|c.is_digit(10)) } else if key.len()==3 && key.ends_with("XX") { matches!(key.chars().next(), Some('1'..='5')) } else { false }
}

// Interop with ObjectElement
impl From<ObjectElement> for ResponsesElement { fn from(obj:ObjectElement)->Self{ ResponsesElement::with_content(obj) }}
impl From<ResponsesElement> for ObjectElement { fn from(el:ResponsesElement)->Self{ el.object }}