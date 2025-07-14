use crate::ast::minim_model::*;

/// OpenAPI Reference Element
#[derive(Debug, Clone)]
pub struct ReferenceElement {
    pub object: ObjectElement,
}

impl ReferenceElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("reference");
        obj.add_class("openapi-reference");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("reference");
        content.add_class("openapi-reference");
        Self { object: content }
    }

    pub fn ref_(&self) -> Option<&StringElement> {
        self.object.get("$ref").and_then(Element::as_string)
    }

    pub fn set_ref(&mut self, value: StringElement) {
        self.object.set("$ref", Element::String(value));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element){ let k=key.into(); self.object.set(&k,value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self,key:&str)->bool{ self.object.has_key(key) }
    pub fn remove(&mut self,key:&str)->Option<Element>{ if let Some(pos)=self.object.content.iter().position(|m|{ if let Element::String(s)=&*m.key{s.content==key}else{false}}){ let m=self.object.content.remove(pos); Some(*m.value) } else { None } }
    pub fn keys(&self)->impl Iterator<Item=&String>{ self.object.content.iter().filter_map(|m|{ if let Element::String(s)=&*m.key{ Some(&s.content) } else { None } }) }
    pub fn values(&self)->impl Iterator<Item=&Element>{ self.object.content.iter().map(|m| m.value.as_ref()) }
    pub fn len(&self)->usize{ self.object.content.len() }
    pub fn is_empty(&self)->bool{ self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self,key:&str)->Option<&Element>{ if key.starts_with("x-"){ self.get_field(key) } else { None } }
    pub fn set_extension(&mut self,key:impl Into<String>,value:Element){ let k=key.into(); if k.starts_with("x-"){ self.set_field(&k,value); } }

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(), String>{ self.ref_().filter(|r|!r.content.trim().is_empty()).map(|_|()).ok_or_else(||"ReferenceElement.$ref is required".into()) }
}

// Interop
impl From<ObjectElement> for ReferenceElement{ fn from(obj:ObjectElement)->Self{ ReferenceElement::with_content(obj) }}
impl From<ReferenceElement> for ObjectElement{ fn from(el:ReferenceElement)->Self{ el.object }}