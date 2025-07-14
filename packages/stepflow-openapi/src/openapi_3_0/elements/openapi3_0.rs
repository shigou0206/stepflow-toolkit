use crate::ast::minim_model::*;

#[derive(Debug, Clone)]
pub struct OpenApi3_0Element {
    pub object: ObjectElement,
}

impl OpenApi3_0Element {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("openApi3_0");
        obj.add_class("api");
        Self { object: obj }
    }

    pub fn with_content(mut content: ObjectElement) -> Self {
        content.set_element_type("openApi3_0");
        content.add_class("api");
        Self { object: content }
    }

    // openapi: string
    pub fn openapi(&self) -> Option<&StringElement> {
        self.object.get("openapi").and_then(Element::as_string)
    }

    pub fn set_openapi(&mut self, val: StringElement) {
        self.object.set("openapi", Element::String(val));
    }

    // info
    pub fn info(&self) -> Option<&ObjectElement> {
        self.object.get("info").and_then(Element::as_object)
    }

    pub fn set_info(&mut self, val: ObjectElement) {
        self.object.set("info", Element::Object(val));
    }

    // servers: Array
    pub fn servers(&self) -> Option<&ArrayElement> {
        self.object.get("servers").and_then(Element::as_array)
    }

    pub fn set_servers(&mut self, val: ArrayElement) {
        self.object.set("servers", Element::Array(val));
    }

    // paths
    pub fn paths(&self) -> Option<&ObjectElement> {
        self.object.get("paths").and_then(Element::as_object)
    }

    pub fn set_paths(&mut self, val: ObjectElement) {
        self.object.set("paths", Element::Object(val));
    }

    // components
    pub fn components(&self) -> Option<&ObjectElement> {
        self.object.get("components").and_then(Element::as_object)
    }

    pub fn set_components(&mut self, val: ObjectElement) {
        self.object.set("components", Element::Object(val));
    }

    // security
    pub fn security(&self) -> Option<&ArrayElement> {
        self.object.get("security").and_then(Element::as_array)
    }

    pub fn set_security(&mut self, val: ArrayElement) {
        self.object.set("security", Element::Array(val));
    }

    // tags
    pub fn tags(&self) -> Option<&ArrayElement> {
        self.object.get("tags").and_then(Element::as_array)
    }

    pub fn set_tags(&mut self, val: ArrayElement) {
        self.object.set("tags", Element::Array(val));
    }

    // externalDocs
    pub fn external_docs(&self) -> Option<&ObjectElement> {
        self.object.get("externalDocs").and_then(Element::as_object)
    }

    pub fn set_external_docs(&mut self, val: ObjectElement) {
        self.object.set("externalDocs", Element::Object(val));
    }
}