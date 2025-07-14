use crate::ast::minim_model::*;

/// OpenAPI Response Content Element
/// Specialized element for response content which is a Map type for media types
#[derive(Debug, Clone)]
pub struct ResponseContentElement {
    pub object: ObjectElement,
}

impl ResponseContentElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("responseContent");
        obj.add_class("response-content");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("responseContent");
        content.add_class("response-content");
        Self { object: content }
    }

    /// Get a media type by name
    pub fn get_media_type(&self, media_type: &str) -> Option<&Element> {
        self.object.get(media_type)
    }

    /// Set a media type
    pub fn set_media_type(&mut self, media_type: &str, element: Element) {
        self.object.set(media_type, element);
    }

    /// Get all media type names
    pub fn media_type_names(&self) -> Vec<String> {
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

    /// Check if content has a specific media type
    pub fn has_media_type(&self, media_type: &str) -> bool {
        self.object.has_key(media_type)
    }

    /// Get the number of media types
    pub fn media_type_count(&self) -> usize {
        self.object.content.len()
    }
}

/// OpenAPI Response Headers Element
/// Specialized element for response headers which is a Map type
#[derive(Debug, Clone)]
pub struct ResponseHeadersElement {
    pub object: ObjectElement,
}

impl ResponseHeadersElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("responseHeaders");
        obj.add_class("response-headers");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("responseHeaders");
        content.add_class("response-headers");
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
}

/// OpenAPI Response Links Element  
/// Specialized element for response links which is a Map type
#[derive(Debug, Clone)]
pub struct ResponseLinksElement {
    pub object: ObjectElement,
}

impl ResponseLinksElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("responseLinks");
        obj.add_class("response-links");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("responseLinks");
        content.add_class("response-links");
        Self { object: content }
    }

    /// Get a link by name
    pub fn get_link(&self, name: &str) -> Option<&Element> {
        self.object.get(name)
    }

    /// Set a link
    pub fn set_link(&mut self, name: &str, link: Element) {
        self.object.set(name, link);
    }

    /// Get all link names
    pub fn link_names(&self) -> Vec<String> {
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

    /// Check if links has a specific name
    pub fn has_link(&self, name: &str) -> bool {
        self.object.has_key(name)
    }

    /// Get the number of links
    pub fn link_count(&self) -> usize {
        self.object.content.len()
    }
}

/// OpenAPI Response Element
#[derive(Debug, Clone)]
pub struct ResponseElement {
    pub object: ObjectElement,
}

impl ResponseElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("response");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("response");
        Self { object: content }
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, value: StringElement) {
        self.object.set("description", Element::String(value));
    }

    pub fn headers(&self) -> Option<&ObjectElement> {
        self.object.get("headers").and_then(Element::as_object)
    }

    pub fn set_headers(&mut self, value: ObjectElement) {
        self.object.set("headers", Element::Object(value));
    }

    /// Set headers using structured ResponseHeadersElement
    pub fn set_response_headers(&mut self, value: ResponseHeadersElement) {
        self.object.set("headers", Element::Object(value.object));
    }

    pub fn content_prop(&self) -> Option<&ObjectElement> {
        self.object.get("content").and_then(Element::as_object)
    }

    pub fn set_content_prop(&mut self, value: ObjectElement) {
        self.object.set("content", Element::Object(value));
    }

    /// Set content using structured ResponseContentElement
    pub fn set_response_content(&mut self, value: ResponseContentElement) {
        self.object.set("content", Element::Object(value.object));
    }

    pub fn links(&self) -> Option<&ObjectElement> {
        self.object.get("links").and_then(Element::as_object)
    }

    pub fn set_links(&mut self, value: ObjectElement) {
        self.object.set("links", Element::Object(value));
    }

    /// Set links using structured ResponseLinksElement
    pub fn set_response_links(&mut self, value: ResponseLinksElement) {
        self.object.set("links", Element::Object(value.object));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element) { let k = key.into(); self.object.set(&k, value); }

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
    pub fn validate_basic(&self)->Result<(), String>{
        if self.description().map(|d| d.content.trim().is_empty()).unwrap_or(true){ return Err("ResponseElement.description is required".into()); }
        // At least one of headers/content/links
        let ok = self.headers().is_some() || self.content_prop().is_some() || self.links().is_some();
        if !ok { return Err("ResponseElement must have at least one of headers, content, or links".into()); }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ResponseElement { fn from(obj:ObjectElement)->Self{ ResponseElement::with_content(obj) }}
impl From<ResponseElement> for ObjectElement { fn from(el:ResponseElement)->Self{ el.object }}