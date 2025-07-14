use crate::ast::minim_model::*;
use crate::openapi_3_0::elements::oauth_flow::OAuthFlowElement;

/// OpenAPI `OAuthFlows` Element
#[derive(Debug, Clone)]
pub struct OAuthFlowsElement {
    pub object: ObjectElement,
}

impl OAuthFlowsElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("oAuthFlows");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("oAuthFlows");
        Self { object: content }
    }

    pub fn implicit(&self) -> Option<OAuthFlowElement> {
        self.object.get("implicit").and_then(Element::as_object).map(|obj| {
            OAuthFlowElement::with_content(obj.clone())
        })
    }

    pub fn set_implicit(&mut self, val: OAuthFlowElement) {
        self.object.set("implicit", Element::Object(val.object));
    }

    pub fn password(&self) -> Option<OAuthFlowElement> {
        self.object.get("password").and_then(Element::as_object).map(|obj| {
            OAuthFlowElement::with_content(obj.clone())
        })
    }

    pub fn set_password(&mut self, val: OAuthFlowElement) {
        self.object.set("password", Element::Object(val.object));
    }

    pub fn client_credentials(&self) -> Option<OAuthFlowElement> {
        self.object.get("clientCredentials").and_then(Element::as_object).map(|obj| {
            OAuthFlowElement::with_content(obj.clone())
        })
    }

    pub fn set_client_credentials(&mut self, val: OAuthFlowElement) {
        self.object.set("clientCredentials", Element::Object(val.object));
    }

    pub fn authorization_code(&self) -> Option<OAuthFlowElement> {
        self.object.get("authorizationCode").and_then(Element::as_object).map(|obj| {
            OAuthFlowElement::with_content(obj.clone())
        })
    }

    pub fn set_authorization_code(&mut self, val: OAuthFlowElement) {
        self.object.set("authorizationCode", Element::Object(val.object));
    }
}