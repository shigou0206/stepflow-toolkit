use crate::ast::minim_model::*;

/// OpenAPI Operation Parameters Element
/// Equivalent to TypeScript OperationParametersElement
#[derive(Debug, Clone)]
pub struct OperationParametersElement {
    pub array: ArrayElement,
}

impl OperationParametersElement {
    pub fn new() -> Self {
        let mut array = ArrayElement::new_empty();
        array.set_element_type("operationParameters");
        array.add_class("operation-parameters");
        Self { array }
    }

    pub fn with_content(content: ArrayElement) -> Self {
        let mut content = content;
        content.set_element_type("operationParameters");
        content.add_class("operation-parameters");
        Self { array: content }
    }

    pub fn push(&mut self, element: Element) {
        self.array.content.push(element);
    }

    pub fn len(&self) -> usize {
        self.array.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.content.is_empty()
    }
}

/// OpenAPI Operation Security Element
/// Equivalent to TypeScript OperationSecurityElement
#[derive(Debug, Clone)]
pub struct OperationSecurityElement {
    pub array: ArrayElement,
}

impl OperationSecurityElement {
    pub fn new() -> Self {
        let mut array = ArrayElement::new_empty();
        array.set_element_type("operationSecurity");
        array.add_class("operation-security");
        Self { array }
    }

    pub fn with_content(content: ArrayElement) -> Self {
        let mut content = content;
        content.set_element_type("operationSecurity");
        content.add_class("operation-security");
        Self { array: content }
    }

    pub fn push(&mut self, element: Element) {
        self.array.content.push(element);
    }

    pub fn len(&self) -> usize {
        self.array.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.content.is_empty()
    }
}

/// OpenAPI Operation Servers Element
/// Equivalent to TypeScript OperationServersElement
#[derive(Debug, Clone)]
pub struct OperationServersElement {
    pub array: ArrayElement,
}

impl OperationServersElement {
    pub fn new() -> Self {
        let mut array = ArrayElement::new_empty();
        array.set_element_type("operationServers");
        array.add_class("operation-servers");
        Self { array }
    }

    pub fn with_content(content: ArrayElement) -> Self {
        let mut content = content;
        content.set_element_type("operationServers");
        content.add_class("operation-servers");
        Self { array: content }
    }

    pub fn push(&mut self, element: Element) {
        self.array.content.push(element);
    }

    pub fn len(&self) -> usize {
        self.array.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.content.is_empty()
    }
}

/// OpenAPI Operation Tags Element
/// Equivalent to TypeScript OperationTagsElement
#[derive(Debug, Clone)]
pub struct OperationTagsElement {
    pub array: ArrayElement,
}

impl OperationTagsElement {
    pub fn new() -> Self {
        let mut array = ArrayElement::new_empty();
        array.set_element_type("operationTags");
        array.add_class("operation-tags");
        Self { array }
    }

    pub fn with_content(content: ArrayElement) -> Self {
        let mut content = content;
        content.set_element_type("operationTags");
        content.add_class("operation-tags");
        Self { array: content }
    }

    pub fn push(&mut self, element: Element) {
        self.array.content.push(element);
    }

    pub fn len(&self) -> usize {
        self.array.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.array.content.is_empty()
    }
}

/// OpenAPI Operation Callbacks Element
/// Equivalent to TypeScript OperationCallbacksElement
#[derive(Debug, Clone)]
pub struct OperationCallbacksElement {
    pub object: ObjectElement,
}

impl OperationCallbacksElement {
    pub fn new() -> Self {
        let mut object = ObjectElement::new();
        object.set_element_type("operationCallbacks");
        object.add_class("operation-callbacks");
        Self { object }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("operationCallbacks");
        content.add_class("operation-callbacks");
        Self { object: content }
    }

    pub fn set(&mut self, key: &str, value: Element) {
        self.object.set(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.object.has_key(key)
    }
}

#[derive(Debug, Clone)]
pub struct OperationElement {
    pub object: ObjectElement,
}

impl OperationElement {
    pub fn new() -> Self {
        let mut object = ObjectElement::new();
        object.set_element_type("operation");
        Self { object }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("operation");
        Self { object: content }
    }

    pub fn summary(&self) -> Option<&StringElement> {
        self.object.get("summary").and_then(Element::as_string)
    }

    pub fn set_summary(&mut self, val: StringElement) {
        self.object.set("summary", Element::String(val));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
    }

    pub fn operation_id(&self) -> Option<&StringElement> {
        self.object.get("operationId").and_then(Element::as_string)
    }

    pub fn set_operation_id(&mut self, val: StringElement) {
        self.object.set("operationId", Element::String(val));
    }

    pub fn parameters(&self) -> Option<&ArrayElement> {
        self.object.get("parameters").and_then(Element::as_array)
    }

    pub fn set_parameters(&mut self, val: ArrayElement) {
        self.object.set("parameters", Element::Array(val));
    }

    /// Set parameters using structured OperationParametersElement
    pub fn set_operation_parameters(&mut self, val: OperationParametersElement) {
        self.object.set("parameters", Element::Array(val.array));
    }

    pub fn request_body(&self) -> Option<&Element> {
        self.object.get("requestBody")
    }

    pub fn set_request_body(&mut self, val: Element) {
        self.object.set("requestBody", val);
    }

    pub fn responses(&self) -> Option<&Element> {
        self.object.get("responses")
    }

    pub fn set_responses(&mut self, val: Element) {
        self.object.set("responses", val);
    }

    pub fn callbacks(&self) -> Option<&ObjectElement> {
        self.object.get("callbacks").and_then(Element::as_object)
    }

    pub fn set_callbacks(&mut self, val: ObjectElement) {
        self.object.set("callbacks", Element::Object(val));
    }

    /// Set callbacks using structured OperationCallbacksElement
    pub fn set_operation_callbacks(&mut self, val: OperationCallbacksElement) {
        self.object.set("callbacks", Element::Object(val.object));
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

    pub fn security(&self) -> Option<&ArrayElement> {
        self.object.get("security").and_then(Element::as_array)
    }

    pub fn set_security(&mut self, val: ArrayElement) {
        self.object.set("security", Element::Array(val));
    }

    /// Set security using structured OperationSecurityElement
    pub fn set_operation_security(&mut self, val: OperationSecurityElement) {
        self.object.set("security", Element::Array(val.array));
    }

    pub fn servers(&self) -> Option<&ArrayElement> {
        self.object.get("servers").and_then(Element::as_array)
    }

    pub fn set_servers(&mut self, val: ArrayElement) {
        self.object.set("servers", Element::Array(val));
    }

    /// Set servers using structured OperationServersElement
    pub fn set_operation_servers(&mut self, val: OperationServersElement) {
        self.object.set("servers", Element::Array(val.array));
    }

    pub fn tags(&self) -> Option<&ArrayElement> {
        self.object.get("tags").and_then(Element::as_array)
    }

    pub fn set_tags(&mut self, val: ArrayElement) {
        self.object.set("tags", Element::Array(val));
    }

    /// Set tags using structured OperationTagsElement
    pub fn set_operation_tags(&mut self, val: OperationTagsElement) {
        self.object.set("tags", Element::Array(val.array));
    }

    pub fn external_docs(&self) -> Option<&ObjectElement> {
        self.object.get("externalDocs").and_then(Element::as_object)
    }

    pub fn set_external_docs(&mut self, val: ObjectElement) {
        self.object.set("externalDocs", Element::Object(val));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element) { let k = key.into(); self.object.set(&k, value); }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }
    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|member| {
            if let Element::String(s) = &*member.key { s.content == key } else { false }
        }) { let member = self.object.content.remove(pos); Some(*member.value) } else { None }
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> { self.object.content.iter().filter_map(|m| { if let Element::String(s)=&*m.key { Some(&s.content) } else { None } }) }
    pub fn values(&self) -> impl Iterator<Item = &Element> { self.object.content.iter().map(|m| m.value.as_ref()) }
    pub fn len(&self) -> usize { self.object.content.len() }
    pub fn is_empty(&self) -> bool { self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key: &str) -> Option<&Element> { if key.starts_with("x-") { self.get_field(key) } else { None } }
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) { let k = key.into(); if k.starts_with("x-") { self.set_field(&k, value); } }

    // -------- Basic validation --------
    /// responses 字段必填且非空，parameters 数组中元素必须为对象/引用
    pub fn validate_basic(&self) -> Result<(), String> {
        // responses must exist
        if let Some(Element::Object(obj)) = self.get_field("responses") {
            if obj.content.is_empty() { return Err("OperationElement.responses must not be empty".to_string()); }
        } else { return Err("OperationElement.responses is required".to_string()); }

        // parameters validation (optional)
        if let Some(Element::Array(arr)) = self.get_field("parameters") {
            for el in &arr.content { match el { Element::Object(_) | Element::Ref(_) => {}, _ => return Err("OperationElement.parameters items must be objects or $ref".to_string()) } }
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for OperationElement { fn from(obj: ObjectElement) -> Self { OperationElement::with_content(obj) } }
impl From<OperationElement> for ObjectElement { fn from(el: OperationElement) -> Self { el.object } }