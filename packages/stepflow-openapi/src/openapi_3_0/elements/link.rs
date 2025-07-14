use crate::ast::minim_model::*;
use serde_json::Value;

/// OpenAPI Link Parameters Element
/// Equivalent to TypeScript LinkParametersElement with MapVisitor pattern
#[derive(Debug, Clone)]
pub struct LinkParametersElement {
    pub object: ObjectElement,
}

impl LinkParametersElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.meta.properties.insert(
            "element-type".to_string(),
            Value::String("linkParameters".to_string())
        );
        obj.classes.content.push(Element::String(StringElement::new("link-parameters")));
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("linkParameters");
        content.classes.content.push(Element::String(StringElement::new("link-parameters")));
        Self { object: content }
    }
}

/// OpenAPI Link Object Element
#[derive(Debug, Clone)]
pub struct LinkElement {
    pub object: ObjectElement,
}

impl LinkElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("link");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("link");
        Self { object: content }
    }

    pub fn operation_ref(&self) -> Option<&StringElement> {
        self.object.get("operationRef").and_then(Element::as_string)
    }

    pub fn set_operation_ref(&mut self, val: StringElement) {
        self.object.set("operationRef", Element::String(val));
    }

    pub fn operation_id(&self) -> Option<&StringElement> {
        self.object.get("operationId").and_then(Element::as_string)
    }

    pub fn set_operation_id(&mut self, val: StringElement) {
        self.object.set("operationId", Element::String(val));
    }

    /// 返回 operation 信息，支持通过 resolver 解析引用
    pub fn operation(&self, resolver: &impl Fn(&str) -> Option<Element>) -> Option<Element> {
        // 首先尝试从 operationRef 的 meta 中获取并解析
        if let Some(op_ref) = self.operation_ref() {
            if let Some(Value::String(ref_path)) = op_ref.meta.properties.get("operation") {
                return resolver(ref_path);
            }
        }

        // 然后尝试从 operationId 的 meta 中获取并解析
        if let Some(op_id) = self.operation_id() {
            if let Some(Value::String(op_name)) = op_id.meta.properties.get("operation") {
                return resolver(op_name);
            }
        }

        // 最后直接从当前对象的字段中获取
        self.object.get("operation").cloned()
    }

    pub fn set_operation(&mut self, val: Element) {
        self.object.set("operation", val);
    }

    pub fn parameters(&self) -> Option<&ObjectElement> {
        self.object.get("parameters").and_then(Element::as_object)
    }

    pub fn set_parameters(&mut self, val: ObjectElement) {
        self.object.set("parameters", Element::Object(val));
    }

    pub fn request_body(&self) -> Option<&Element> {
        self.object.get("requestBody")
    }

    pub fn set_request_body(&mut self, val: Element) {
        self.object.set("requestBody", val);
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
    }

    pub fn server(&self) -> Option<&ObjectElement> {
        self.object.get("server").and_then(Element::as_object)
    }

    pub fn set_server(&mut self, val: ObjectElement) {
        self.object.set("server", Element::Object(val));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element){ let k=key.into(); self.object.set(&k,value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self,key:&str)->bool{ self.object.has_key(key) }
    pub fn remove(&mut self,key:&str)->Option<Element>{ if let Some(pos)=self.object.content.iter().position(|m|{ if let Element::String(s)=&*m.key{ s.content==key }else{ false }}){ let m=self.object.content.remove(pos); Some(*m.value) }else{ None } }
    pub fn keys(&self)->impl Iterator<Item=&String>{ self.object.content.iter().filter_map(|m|{ if let Element::String(s)=&*m.key{ Some(&s.content) }else{ None } }) }
    pub fn values(&self)->impl Iterator<Item=&Element>{ self.object.content.iter().map(|m| m.value.as_ref()) }
    pub fn len(&self)->usize{ self.object.content.len() }
    pub fn is_empty(&self)->bool{ self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self,key:&str)->Option<&Element>{ if key.starts_with("x-"){ self.get_field(key) }else{ None } }
    pub fn set_extension(&mut self,key:impl Into<String>,value:Element){ let k=key.into(); if k.starts_with("x-"){ self.set_field(&k,value); } }

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(), String>{
        let has_ref=self.operation_ref().is_some();
        let has_id=self.operation_id().is_some();
        if has_ref==has_id { // both true or both false
            return Err("LinkElement must contain exactly one of operationRef or operationId".into());
        }
        Ok(())
    }
}

/// OpenAPI Link Server Element
#[derive(Debug, Clone)]
pub struct LinkServerElement {
    pub object: ObjectElement,
}

impl LinkServerElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.meta.properties.insert(
            "element-type".to_string(),
            Value::String("linkServer".to_string())
        );
        Self { object: obj }
    }
}

// Interop conversions
impl From<ObjectElement> for LinkElement { fn from(obj:ObjectElement)->Self{ LinkElement::with_content(obj) }}
impl From<LinkElement> for ObjectElement { fn from(el:LinkElement)->Self{ el.object }}

impl From<ObjectElement> for LinkParametersElement { fn from(obj:ObjectElement)->Self{ LinkParametersElement::with_content(obj) }}
impl From<LinkParametersElement> for ObjectElement { fn from(el:LinkParametersElement)->Self{ el.object }}