use crate::ast::minim_model::*;

/// OpenAPI Xml Element
#[derive(Debug, Clone)]
pub struct XmlElement {
    pub object: ObjectElement,
}

impl XmlElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("xml");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("xml");
        Self { object: content }
    }

    pub fn name(&self) -> Option<&StringElement> {
        self.object.get("name").and_then(Element::as_string)
    }

    pub fn set_name(&mut self, value: StringElement) {
        self.object.set("name", Element::String(value));
    }

    pub fn namespace(&self) -> Option<&StringElement> {
        self.object.get("namespace").and_then(Element::as_string)
    }

    pub fn set_namespace(&mut self, value: StringElement) {
        self.object.set("namespace", Element::String(value));
    }

    pub fn prefix(&self) -> Option<&StringElement> {
        self.object.get("prefix").and_then(Element::as_string)
    }

    pub fn set_prefix(&mut self, value: StringElement) {
        self.object.set("prefix", Element::String(value));
    }

    pub fn attribute(&self) -> Option<&BooleanElement> {
        self.object.get("attribute").and_then(Element::as_boolean)
    }

    pub fn set_attribute(&mut self, value: BooleanElement) {
        self.object.set("attribute", Element::Boolean(value));
    }

    pub fn wrapped(&self) -> Option<&BooleanElement> {
        self.object.get("wrapped").and_then(Element::as_boolean)
    }

    pub fn set_wrapped(&mut self, value: BooleanElement) {
        self.object.set("wrapped", Element::Boolean(value));
    }
}