use crate::ast::minim_model::*;

/// OpenAPI Discriminator Element
#[derive(Debug, Clone)]
pub struct DiscriminatorElement {
    pub object: ObjectElement,
}

impl DiscriminatorElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("discriminator");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("discriminator");
        Self { object: content }
    }

    pub fn property_name(&self) -> Option<&StringElement> {
        self.object.get("propertyName").and_then(Element::as_string)
    }

    pub fn set_property_name(&mut self, value: StringElement) {
        self.object.set("propertyName", Element::String(value));
    }

    pub fn mapping(&self) -> Option<&ObjectElement> {
        self.object.get("mapping").and_then(Element::as_object)
    }

    pub fn set_mapping(&mut self, value: ObjectElement) {
        self.object.set("mapping", Element::Object(value));
    }
}