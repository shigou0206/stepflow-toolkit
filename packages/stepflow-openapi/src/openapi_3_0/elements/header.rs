use crate::ast::minim_model::*;

/// OpenAPI Header Element
#[derive(Debug, Clone)]
pub struct HeaderElement {
    pub object: ObjectElement,
}

impl HeaderElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("header");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("header");
        Self { object: content }
    }

    pub fn required(&self) -> bool {
        self.object
            .get("required")
            .and_then(Element::as_boolean)
            .map(|b| b.content)
            .unwrap_or(false)
    }

    pub fn set_required(&mut self, value: bool) {
        self.object.set("required", Element::Boolean(BooleanElement::new(value)));
    }

    pub fn deprecated(&self) -> bool {
        self.object
            .get("deprecated")
            .and_then(Element::as_boolean)
            .map(|b| b.content)
            .unwrap_or(false)
    }

    pub fn set_deprecated(&mut self, value: bool) {
        self.object.set("deprecated", Element::Boolean(BooleanElement::new(value)));
    }

    pub fn allow_empty_value(&self) -> Option<&BooleanElement> {
        self.object.get("allowEmptyValue").and_then(Element::as_boolean)
    }

    pub fn set_allow_empty_value(&mut self, value: BooleanElement) {
        self.object.set("allowEmptyValue", Element::Boolean(value));
    }

    pub fn style(&self) -> Option<&StringElement> {
        self.object.get("style").and_then(Element::as_string)
    }

    pub fn set_style(&mut self, val: StringElement) {
        self.object.set("style", Element::String(val));
    }

    pub fn explode(&self) -> Option<&BooleanElement> {
        self.object.get("explode").and_then(Element::as_boolean)
    }

    pub fn set_explode(&mut self, value: BooleanElement) {
        self.object.set("explode", Element::Boolean(value));
    }

    pub fn allow_reserved(&self) -> Option<&BooleanElement> {
        self.object.get("allowReserved").and_then(Element::as_boolean)
    }

    pub fn set_allow_reserved(&mut self, value: BooleanElement) {
        self.object.set("allowReserved", Element::Boolean(value));
    }

    pub fn schema(&self) -> Option<&Element> {
        self.object.get("schema")
    }

    pub fn set_schema(&mut self, val: Element) {
        self.object.set("schema", val);
    }

    pub fn example(&self) -> Option<&Element> {
        self.object.get("example")
    }

    pub fn set_example(&mut self, val: Element) {
        self.object.set("example", val);
    }

    pub fn examples(&self) -> Option<&ObjectElement> {
        self.object.get("examples").and_then(Element::as_object)
    }

    pub fn set_examples(&mut self, val: ObjectElement) {
        self.object.set("examples", Element::Object(val));
    }

    pub fn content(&self) -> Option<&ObjectElement> {
        self.object.get("content").and_then(Element::as_object)
    }

    pub fn set_content(&mut self, val: ObjectElement) {
        self.object.set("content", Element::Object(val));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
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
    pub fn validate_basic(&self)->Result<(), String>{
        // style must be simple if present
        if let Some(style)=self.style(){ if style.content.as_str()!="simple"{ return Err("HeaderElement.style must be 'simple'".into()); } }
        // Cannot have both schema and content, must have at least one
        let has_schema=self.schema().is_some();
        let has_content=self.content().map(|o|!o.content.is_empty()).unwrap_or(false);
        if has_schema && has_content { return Err("HeaderElement cannot have both schema and content".into()); }
        if !has_schema && !has_content { return Err("HeaderElement must have either schema or content".into()); }
        Ok(())
    }
}

// Interop
impl From<ObjectElement> for HeaderElement{ fn from(obj:ObjectElement)->Self{ HeaderElement::with_content(obj) }}
impl From<HeaderElement> for ObjectElement{ fn from(el:HeaderElement)->Self{ el.object }}