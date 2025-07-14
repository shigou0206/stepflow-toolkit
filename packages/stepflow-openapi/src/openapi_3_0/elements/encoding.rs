use crate::ast::minim_model::*;

/// OpenAPI Encoding Element
#[derive(Debug, Clone)]
pub struct EncodingElement {
    pub object: ObjectElement,
}

impl EncodingElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("encoding");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("encoding");
        Self { object: content }
    }

    pub fn content_type(&self) -> Option<&StringElement> {
        self.object.get("contentType").and_then(Element::as_string)
    }

    pub fn set_content_type(&mut self, value: StringElement) {
        self.object.set("contentType", Element::String(value));
    }

    pub fn headers(&self) -> Option<&ObjectElement> {
        self.object.get("headers").and_then(Element::as_object)
    }

    pub fn set_headers(&mut self, value: ObjectElement) {
        self.object.set("headers", Element::Object(value));
    }

    pub fn style(&self) -> Option<&StringElement> {
        self.object.get("style").and_then(Element::as_string)
    }

    pub fn set_style(&mut self, value: StringElement) {
        self.object.set("style", Element::String(value));
    }

    pub fn explode(&self) -> Option<&BooleanElement> {
        self.object.get("explode").and_then(Element::as_boolean)
    }

    pub fn set_explode(&mut self, value: BooleanElement) {
        self.object.set("explode", Element::Boolean(value));
    }

    pub fn allowed_reserved(&self) -> Option<&BooleanElement> {
        self.object.get("allowedReserved").and_then(Element::as_boolean)
    }

    pub fn set_allowed_reserved(&mut self, value: BooleanElement) {
        self.object.set("allowedReserved", Element::Boolean(value));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element) { let k=key.into(); self.object.set(&k,value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }
    pub fn remove(&mut self, key: &str) -> Option<Element> { if let Some(pos)=self.object.content.iter().position(|m|{if let Element::String(s)=&*m.key{s.content==key}else{false}}){let m=self.object.content.remove(pos);Some(*m.value)}else{None} }
    pub fn keys(&self)->impl Iterator<Item=&String>{self.object.content.iter().filter_map(|m|{if let Element::String(s)=&*m.key{Some(&s.content)}else{None}})}
    pub fn values(&self)->impl Iterator<Item=&Element>{self.object.content.iter().map(|m|m.value.as_ref())}
    pub fn len(&self)->usize{self.object.content.len()}
    pub fn is_empty(&self)->bool{self.object.content.is_empty()}

    // -------- Extension helpers --------
    pub fn get_extension(&self, key:&str)->Option<&Element>{if key.starts_with("x-"){self.get_field(key)}else{None}}
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element){let k=key.into(); if k.starts_with("x-"){self.set_field(&k,value);} }

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(),String>{
        if let Some(ct)=self.content_type(){ if ct.content.trim().is_empty(){return Err("EncodingElement.contentType cannot be empty".into());}}
        // style if present must be one of form/simple
        if let Some(style) = self.style() {
            match style.content.as_str() {
                "form" | "simple" => {}
                _ => {
                    return Err(format!(
                        "Invalid encoding style: {}",
                        style.content
                    ))
                }
            }
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for EncodingElement{fn from(obj:ObjectElement)->Self{EncodingElement::with_content(obj)}}
impl From<EncodingElement> for ObjectElement{fn from(el:EncodingElement)->Self{el.object}}