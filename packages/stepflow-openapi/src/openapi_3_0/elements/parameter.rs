use crate::ast::minim_model::*;

/// OpenAPI Parameter Element
#[derive(Debug, Clone)]
pub struct ParameterElement {
    pub object: ObjectElement,
}

impl ParameterElement {
    pub fn new() -> Self {
        let mut object = ObjectElement::new();
        object.set_element_type("parameter");
        Self { object }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("parameter");
        Self { object: content }
    }

    pub fn name(&self) -> Option<&StringElement> {
        self.object.get("name").and_then(Element::as_string)
    }

    pub fn set_name(&mut self, val: StringElement) {
        self.object.set("name", Element::String(val));
    }

    pub fn in_(&self) -> Option<&StringElement> {
        self.object.get("in").and_then(Element::as_string)
    }

    pub fn set_in(&mut self, val: StringElement) {
        self.object.set("in", Element::String(val));
    }

    pub fn required(&self) -> bool {
        self.object
            .get("required")
            .and_then(Element::as_boolean)
            .map(|b| b.content)
            .unwrap_or(false)
    }

    pub fn set_required(&mut self, val: bool) {
        self.object.set("required", Element::Boolean(BooleanElement::new(val)));
    }

    pub fn deprecated(&self) -> bool {
        self.object
            .get("deprecated")
            .and_then(Element::as_boolean)
            .map(|b| b.content)
            .unwrap_or(false)
    }

    pub fn set_deprecated(&mut self, val: bool) {
        self.object.set("deprecated", Element::Boolean(BooleanElement::new(val)));
    }

    pub fn allow_empty_value(&self) -> Option<&BooleanElement> {
        self.object.get("allowEmptyValue").and_then(Element::as_boolean)
    }

    pub fn set_allow_empty_value(&mut self, val: BooleanElement) {
        self.object.set("allowEmptyValue", Element::Boolean(val));
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

    pub fn set_explode(&mut self, val: BooleanElement) {
        self.object.set("explode", Element::Boolean(val));
    }

    pub fn allow_reserved(&self) -> Option<&BooleanElement> {
        self.object.get("allowReserved").and_then(Element::as_boolean)
    }

    pub fn set_allow_reserved(&mut self, val: BooleanElement) {
        self.object.set("allowReserved", Element::Boolean(val));
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

    pub fn content_prop(&self) -> Option<&ObjectElement> {
        self.object.get("content").and_then(Element::as_object)
    }

    pub fn set_content_prop(&mut self, val: ObjectElement) {
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
    pub fn set_field(&mut self, key: impl Into<String>, value: Element) { let k=key.into(); self.object.set(&k,value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self,key:&str)->bool{self.object.has_key(key)}
    pub fn remove(&mut self,key:&str)->Option<Element>{if let Some(pos)=self.object.content.iter().position(|m|{if let Element::String(s)=&*m.key{s.content==key}else{false}}){let m=self.object.content.remove(pos);Some(*m.value)}else{None}}
    pub fn keys(&self)->impl Iterator<Item=&String>{self.object.content.iter().filter_map(|m|{if let Element::String(s)=&*m.key{Some(&s.content)}else{None}})}
    pub fn values(&self)->impl Iterator<Item=&Element>{self.object.content.iter().map(|m|m.value.as_ref())}
    pub fn len(&self)->usize{self.object.content.len()}
    pub fn is_empty(&self)->bool{self.object.content.is_empty()}

    // -------- Extension helpers --------
    pub fn get_extension(&self,key:&str)->Option<&Element>{if key.starts_with("x-"){self.get_field(key)}else{None}}
    pub fn set_extension(&mut self,key:impl Into<String>,value:Element){let k=key.into();if k.starts_with("x-"){self.set_field(&k,value);}}

    // -------- Basic validation --------
    pub fn validate_basic(&self)->Result<(),String>{
        // name and in
        let name_ok=self.name().map(|n|!n.content.trim().is_empty()).unwrap_or(false);
        if !name_ok{return Err("ParameterElement.name is required".into());}
        let location=self.in_().ok_or("ParameterElement.in is required".to_string())?;
        let loc_str = &location.content;
        match loc_str.as_str() {
            "path" | "query" | "header" | "cookie" => {}
            _ => return Err(format!("Invalid parameter location: {}", loc_str)),
        }
        if loc_str=="path" && !self.required(){return Err("Path parameters must be required".into());}
        // either schema or content
        let has_schema=self.schema().is_some();
        let has_content=self.content_prop().map(|o|!o.content.is_empty()).unwrap_or(false);
        if has_schema && has_content{return Err("Parameter cannot have both schema and content".into());}
        if !has_schema && !has_content{return Err("Parameter must have either schema or content".into());}
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ParameterElement{fn from(obj:ObjectElement)->Self{ParameterElement::with_content(obj)}}
impl From<ParameterElement> for ObjectElement{fn from(el:ParameterElement)->Self{el.object}}