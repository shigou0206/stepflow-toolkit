use crate::ast::minim_model::*;

/// OpenAPI `OAuthFlow` Element
#[derive(Debug, Clone)]
pub struct OAuthFlowElement {
    pub object: ObjectElement,
}

impl OAuthFlowElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("oAuthFlow");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("oAuthFlow");
        Self { object: content }
    }

    pub fn authorization_url(&self) -> Option<&StringElement> {
        self.object.get("authorizationUrl").and_then(Element::as_string)
    }

    pub fn set_authorization_url(&mut self, val: StringElement) {
        self.object.set("authorizationUrl", Element::String(val));
    }

    pub fn token_url(&self) -> Option<&StringElement> {
        self.object.get("tokenUrl").and_then(Element::as_string)
    }

    pub fn set_token_url(&mut self, val: StringElement) {
        self.object.set("tokenUrl", Element::String(val));
    }

    pub fn refresh_url(&self) -> Option<&StringElement> {
        self.object.get("refreshUrl").and_then(Element::as_string)
    }

    pub fn set_refresh_url(&mut self, val: StringElement) {
        self.object.set("refreshUrl", Element::String(val));
    }

    pub fn scopes(&self) -> Option<&ObjectElement> {
        self.object.get("scopes").and_then(Element::as_object)
    }

    pub fn set_scopes(&mut self, val: ObjectElement) {
        self.object.set("scopes", Element::Object(val));
    }
}