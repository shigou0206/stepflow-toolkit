use crate::ast::minim_model::*;

/// OpenAPI 3.x Components Element
#[derive(Debug, Clone)]
pub struct ComponentsElement {
    pub object: ObjectElement,
}

impl ComponentsElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("components");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("components");
        Self { object: content }
    }

    // ---------- Accessors ----------

    pub fn schemas(&self) -> Option<&ObjectElement> {
        self.object.get("schemas").and_then(Element::as_object)
    }

    pub fn set_schemas(&mut self, value: ObjectElement) {
        self.object.set("schemas", Element::Object(value));
    }

    pub fn responses(&self) -> Option<&ObjectElement> {
        self.object.get("responses").and_then(Element::as_object)
    }

    pub fn set_responses(&mut self, value: ObjectElement) {
        self.object.set("responses", Element::Object(value));
    }

    pub fn parameters(&self) -> Option<&ObjectElement> {
        self.object.get("parameters").and_then(Element::as_object)
    }

    pub fn set_parameters(&mut self, value: ObjectElement) {
        self.object.set("parameters", Element::Object(value));
    }

    pub fn examples(&self) -> Option<&ObjectElement> {
        self.object.get("examples").and_then(Element::as_object)
    }

    pub fn set_examples(&mut self, value: ObjectElement) {
        self.object.set("examples", Element::Object(value));
    }

    pub fn request_bodies(&self) -> Option<&ObjectElement> {
        self.object.get("requestBodies").and_then(Element::as_object)
    }

    pub fn set_request_bodies(&mut self, value: ObjectElement) {
        self.object.set("requestBodies", Element::Object(value));
    }

    pub fn headers(&self) -> Option<&ObjectElement> {
        self.object.get("headers").and_then(Element::as_object)
    }

    pub fn set_headers(&mut self, value: ObjectElement) {
        self.object.set("headers", Element::Object(value));
    }

    pub fn security_schemes(&self) -> Option<&ObjectElement> {
        self.object.get("securitySchemes").and_then(Element::as_object)
    }

    pub fn set_security_schemes(&mut self, value: ObjectElement) {
        self.object.set("securitySchemes", Element::Object(value));
    }

    pub fn links(&self) -> Option<&ObjectElement> {
        self.object.get("links").and_then(Element::as_object)
    }

    pub fn set_links(&mut self, value: ObjectElement) {
        self.object.set("links", Element::Object(value));
    }

    pub fn callbacks(&self) -> Option<&ObjectElement> {
        self.object.get("callbacks").and_then(Element::as_object)
    }

    pub fn set_callbacks(&mut self, value: ObjectElement) {
        self.object.set("callbacks", Element::Object(value));
    }

    // -------- Generic accessors --------
    pub fn get(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        self.object.set(&k, value);
    }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }

    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|member| {
            if let Element::String(s) = &*member.key { s.content == key } else { false }
        }) {
            let member = self.object.content.remove(pos);
            Some(*member.value)
        } else { None }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.object.content.iter().filter_map(|m| {
            if let Element::String(s) = &*m.key { Some(&s.content) } else { None }
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &Element> {
        self.object.content.iter().map(|m| m.value.as_ref())
    }

    pub fn len(&self) -> usize { self.object.content.len() }
    pub fn is_empty(&self) -> bool { self.object.content.is_empty() }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key: &str) -> Option<&Element> {
        if key.starts_with("x-") { self.get(key) } else { None }
    }

    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into(); if k.starts_with("x-") { self.set(&k, value); }
    }

    // -------- Basic validation --------
    /// 至少包含一种子组件，且各子组件键下的对象不能为空
    pub fn validate_basic(&self) -> Result<(), String> {
        let mut has_any = false;
        for field in ["schemas", "responses", "parameters", "examples", "requestBodies", "headers", "securitySchemes", "links", "callbacks"].iter() {
            if let Some(obj) = self.get(field).and_then(Element::as_object) {
                if !obj.content.is_empty() {
                    has_any = true;
                }
            }
        }
        if has_any { Ok(()) } else { Err("ComponentsElement must contain at least one non-empty sub-component".to_string()) }
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for ComponentsElement {
    fn from(obj: ObjectElement) -> Self { ComponentsElement::with_content(obj) }
}

impl From<ComponentsElement> for ObjectElement {
    fn from(el: ComponentsElement) -> Self { el.object }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_components_element_new() {
        let components = ComponentsElement::new();
        assert_eq!(components.object.element, "components");
        assert!(components.object.content.is_empty());
    }

    #[test]
    fn test_components_element_with_content() {
        let mut obj = ObjectElement::new();
        obj.set("schemas", Element::Object(ObjectElement::new()));
        
        let components = ComponentsElement::with_content(obj);
        assert_eq!(components.object.element, "components");
        assert!(components.schemas().is_some());
    }

    #[test]
    fn test_schemas_get_set() {
        let mut components = ComponentsElement::new();
        
        // 初始状态应该为空
        assert!(components.schemas().is_none());
        
        // 设置 schemas
        let mut schemas_obj = ObjectElement::new();
        schemas_obj.set("User", Element::Object(ObjectElement::new()));
        components.set_schemas(schemas_obj);
        
        // 验证设置成功
        let schemas = components.schemas();
        assert!(schemas.is_some());
        assert!(schemas.unwrap().get("User").is_some());
    }

    #[test]
    fn test_responses_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.responses().is_none());
        
        let mut responses_obj = ObjectElement::new();
        responses_obj.set("NotFound", Element::Object(ObjectElement::new()));
        components.set_responses(responses_obj);
        
        let responses = components.responses();
        assert!(responses.is_some());
        assert!(responses.unwrap().get("NotFound").is_some());
    }

    #[test]
    fn test_parameters_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.parameters().is_none());
        
        let mut params_obj = ObjectElement::new();
        params_obj.set("limit", Element::Object(ObjectElement::new()));
        components.set_parameters(params_obj);
        
        let parameters = components.parameters();
        assert!(parameters.is_some());
        assert!(parameters.unwrap().get("limit").is_some());
    }

    #[test]
    fn test_examples_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.examples().is_none());
        
        let mut examples_obj = ObjectElement::new();
        examples_obj.set("user_example", Element::Object(ObjectElement::new()));
        components.set_examples(examples_obj);
        
        let examples = components.examples();
        assert!(examples.is_some());
        assert!(examples.unwrap().get("user_example").is_some());
    }

    #[test]
    fn test_request_bodies_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.request_bodies().is_none());
        
        let mut request_bodies_obj = ObjectElement::new();
        request_bodies_obj.set("UserBody", Element::Object(ObjectElement::new()));
        components.set_request_bodies(request_bodies_obj);
        
        let request_bodies = components.request_bodies();
        assert!(request_bodies.is_some());
        assert!(request_bodies.unwrap().get("UserBody").is_some());
    }

    #[test]
    fn test_headers_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.headers().is_none());
        
        let mut headers_obj = ObjectElement::new();
        headers_obj.set("X-Rate-Limit", Element::Object(ObjectElement::new()));
        components.set_headers(headers_obj);
        
        let headers = components.headers();
        assert!(headers.is_some());
        assert!(headers.unwrap().get("X-Rate-Limit").is_some());
    }

    #[test]
    fn test_security_schemes_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.security_schemes().is_none());
        
        let mut security_obj = ObjectElement::new();
        security_obj.set("bearerAuth", Element::Object(ObjectElement::new()));
        components.set_security_schemes(security_obj);
        
        let security_schemes = components.security_schemes();
        assert!(security_schemes.is_some());
        assert!(security_schemes.unwrap().get("bearerAuth").is_some());
    }

    #[test]
    fn test_links_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.links().is_none());
        
        let mut links_obj = ObjectElement::new();
        links_obj.set("UserLink", Element::Object(ObjectElement::new()));
        components.set_links(links_obj);
        
        let links = components.links();
        assert!(links.is_some());
        assert!(links.unwrap().get("UserLink").is_some());
    }

    #[test]
    fn test_callbacks_get_set() {
        let mut components = ComponentsElement::new();
        
        assert!(components.callbacks().is_none());
        
        let mut callbacks_obj = ObjectElement::new();
        callbacks_obj.set("myCallback", Element::Object(ObjectElement::new()));
        components.set_callbacks(callbacks_obj);
        
        let callbacks = components.callbacks();
        assert!(callbacks.is_some());
        assert!(callbacks.unwrap().get("myCallback").is_some());
    }

    #[test]
    fn test_multiple_components_together() {
        let mut components = ComponentsElement::new();
        
        // 设置多个组件类型
        let mut schemas = ObjectElement::new();
        schemas.set("User", Element::String(StringElement::new("user_schema")));
        components.set_schemas(schemas);
        
        let mut responses = ObjectElement::new();
        responses.set("Error", Element::String(StringElement::new("error_response")));
        components.set_responses(responses);
        
        let mut security = ObjectElement::new();
        security.set("oauth2", Element::String(StringElement::new("oauth_scheme")));
        components.set_security_schemes(security);
        
        // 验证所有组件都设置成功
        assert!(components.schemas().is_some());
        assert!(components.responses().is_some());
        assert!(components.security_schemes().is_some());
        
        // 验证具体内容
        if let Some(Element::String(s)) = components.schemas().unwrap().get("User") {
            assert_eq!(s.content, "user_schema");
        } else {
            panic!("Expected User schema");
        }
        
        if let Some(Element::String(s)) = components.responses().unwrap().get("Error") {
            assert_eq!(s.content, "error_response");
        } else {
            panic!("Expected Error response");
        }
        
        if let Some(Element::String(s)) = components.security_schemes().unwrap().get("oauth2") {
            assert_eq!(s.content, "oauth_scheme");
        } else {
            panic!("Expected oauth2 security scheme");
        }
    }

    #[test]
    fn test_openapi_components_realistic_scenario() {
        let mut components = ComponentsElement::new();
        
        // 创建一个真实的 User 模式
        let mut user_schema = ObjectElement::new();
        user_schema.set("type", Element::String(StringElement::new("object")));
        
        let mut properties = ObjectElement::new();
        let mut id_prop = ObjectElement::new();
        id_prop.set("type", Element::String(StringElement::new("integer")));
        properties.set("id", Element::Object(id_prop));
        
        let mut name_prop = ObjectElement::new();
        name_prop.set("type", Element::String(StringElement::new("string")));
        properties.set("name", Element::Object(name_prop));
        
        user_schema.set("properties", Element::Object(properties));
        
        // 设置到 components
        let mut schemas = ObjectElement::new();
        schemas.set("User", Element::Object(user_schema));
        components.set_schemas(schemas);
        
        // 验证复杂的嵌套结构
        let schemas = components.schemas().unwrap();
        let user = schemas.get("User").unwrap().as_object().unwrap();
        let props = user.get("properties").unwrap().as_object().unwrap();
        let id = props.get("id").unwrap().as_object().unwrap();
        
        if let Some(Element::String(type_str)) = id.get("type") {
            assert_eq!(type_str.content, "integer");
        } else {
            panic!("Expected integer type for id");
        }
    }
}