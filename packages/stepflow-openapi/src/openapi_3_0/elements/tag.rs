use crate::ast::minim_model::*;
use crate::openapi_3_0::elements::external_documentation::ExternalDocumentationElement;

/// OpenAPI Tag Element
#[derive(Debug, Clone)]
pub struct TagElement {
    pub object: ObjectElement,
}

impl TagElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("tag");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("tag");
        Self { object: content }
    }

    pub fn name(&self) -> Option<&StringElement> {
        self.object.get("name").and_then(Element::as_string)
    }

    pub fn set_name(&mut self, value: StringElement) {
        self.object.set("name", Element::String(value));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, value: StringElement) {
        self.object.set("description", Element::String(value));
    }

    pub fn external_docs(&self) -> Option<ExternalDocumentationElement> {
        self.object
            .get("externalDocs")
            .and_then(Element::as_object)
            .map(|obj| ExternalDocumentationElement::with_content(obj.clone()))
    }

    pub fn set_external_docs(&mut self, value: ExternalDocumentationElement) {
        self.object.set("externalDocs", Element::Object(value.object));
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
    pub fn validate_basic(&self)->Result<(), String>{ if self.name().map(|n|!n.content.trim().is_empty()).unwrap_or(false){ Ok(()) } else { Err("TagElement.name is required".into()) } }
}

// Interop
impl From<ObjectElement> for TagElement{ fn from(obj:ObjectElement)->Self{ TagElement::with_content(obj) }}
impl From<TagElement> for ObjectElement{ fn from(el:TagElement)->Self{ el.object }}