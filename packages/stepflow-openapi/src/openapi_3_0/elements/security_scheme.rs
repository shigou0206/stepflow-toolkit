use crate::ast::minim_model::*;

/// OpenAPI SecurityScheme Element
#[derive(Debug, Clone)]
pub struct SecuritySchemeElement {
    pub object: ObjectElement,
}

impl SecuritySchemeElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("securityScheme");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("securityScheme");
        Self { object: content }
    }

    pub fn type_(&self) -> Option<&StringElement> {
        self.object.get("type").and_then(Element::as_string)
    }

    pub fn set_type(&mut self, value: StringElement) {
        self.object.set("type", Element::String(value));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, value: StringElement) {
        self.object.set("description", Element::String(value));
    }

    pub fn name(&self) -> Option<&StringElement> {
        self.object.get("name").and_then(Element::as_string)
    }

    pub fn set_name(&mut self, value: StringElement) {
        self.object.set("name", Element::String(value));
    }

    pub fn in_(&self) -> Option<&StringElement> {
        self.object.get("in").and_then(Element::as_string)
    }

    pub fn set_in(&mut self, value: StringElement) {
        self.object.set("in", Element::String(value));
    }

    pub fn scheme(&self) -> Option<&StringElement> {
        self.object.get("scheme").and_then(Element::as_string)
    }

    pub fn set_scheme(&mut self, value: StringElement) {
        self.object.set("scheme", Element::String(value));
    }

    pub fn bearer_format(&self) -> Option<&StringElement> {
        self.object.get("bearerFormat").and_then(Element::as_string)
    }

    pub fn set_bearer_format(&mut self, value: StringElement) {
        self.object.set("bearerFormat", Element::String(value));
    }

    pub fn flows(&self) -> Option<&ObjectElement> {
        self.object.get("flows").and_then(Element::as_object)
    }

    pub fn set_flows(&mut self, value: ObjectElement) {
        self.object.set("flows", Element::Object(value));
    }

    pub fn openid_connect_url(&self) -> Option<&StringElement> {
        self.object.get("openIdConnectUrl").and_then(Element::as_string)
    }

    pub fn set_openid_connect_url(&mut self, value: StringElement) {
        self.object.set("openIdConnectUrl", Element::String(value));
    }
}