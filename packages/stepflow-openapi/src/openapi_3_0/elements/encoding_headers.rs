use crate::ast::minim_model::*;

/// OpenAPI Encoding Headers Element
/// Specialized element for encoding headers which is a Map type rather than generic Object
#[derive(Debug, Clone)]
pub struct EncodingHeadersElement {
    pub object: ObjectElement,
}

impl EncodingHeadersElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("encodingHeaders");
        obj.add_class("encoding-headers");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("encodingHeaders");
        content.add_class("encoding-headers");
        Self { object: content }
    }

    /// Get a header by name
    pub fn get_header(&self, name: &str) -> Option<&Element> {
        self.object.get(name)
    }

    /// Set a header
    pub fn set_header(&mut self, name: &str, header: Element) {
        self.object.set(name, header);
    }

    /// Get all header names
    pub fn header_names(&self) -> Vec<String> {
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

    /// Check if headers has a specific name
    pub fn has_header(&self, name: &str) -> bool {
        self.object.has_key(name)
    }

    /// Get the number of headers
    pub fn header_count(&self) -> usize {
        self.object.content.len()
    }

    /// Iterate over all headers
    pub fn headers(&self) -> impl Iterator<Item = (&str, &Element)> {
        self.object.content.iter().filter_map(|member| {
            if let Element::String(key) = &*member.key {
                Some((key.content.as_str(), &*member.value))
            } else {
                None
            }
        })
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element){let k=key.into(); self.object.set(&k,value);}    

    // -------- Convenience helpers --------
    pub fn remove(&mut self, key:&str)->Option<Element>{ if let Some(pos)=self.object.content.iter().position(|m|{if let Element::String(s)=&*m.key{s.content==key}else{false}}){let m=self.object.content.remove(pos);Some(*m.value)}else{None}}
    pub fn len(&self)->usize{self.object.content.len()}
    pub fn is_empty(&self)->bool{self.object.content.is_empty()}

    // -------- Extension helpers --------
    pub fn get_extension(&self,key:&str)->Option<&Element>{if key.starts_with("x-"){self.get_field(key)}else{None}}
    pub fn set_extension(&mut self,key:impl Into<String>,value:Element){let k=key.into(); if k.starts_with("x-"){self.set_field(&k,value);} }

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(),String>{ if self.is_empty(){Err("EncodingHeadersElement must contain at least one header".into())}else{Ok(())} }
}

// Interop with ObjectElement
impl From<ObjectElement> for EncodingHeadersElement{fn from(obj:ObjectElement)->Self{EncodingHeadersElement::with_content(obj)}}
impl From<EncodingHeadersElement> for ObjectElement{fn from(el:EncodingHeadersElement)->Self{el.object}} 