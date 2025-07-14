use crate::ast::minim_model::*;

/// OpenAPI RequestBody Element
#[derive(Debug, Clone)]
pub struct RequestBodyElement {
    pub object: ObjectElement,
}

impl RequestBodyElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("requestBody");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("requestBody");
        Self { object: content }
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, value: StringElement) {
        self.object.set("description", Element::String(value));
    }

    pub fn content_prop(&self) -> Option<&ObjectElement> {
        self.object.get("content").and_then(Element::as_object)
    }

    pub fn set_content_prop(&mut self, value: ObjectElement) {
        self.object.set("content", Element::Object(value));
    }

    pub fn required(&self) -> bool {
        self.object
            .get("required")
            .and_then(Element::as_boolean)
            .map(|b| b.content)
            .unwrap_or(false)
    }

    pub fn set_required(&mut self, value: bool) {
        self.object
            .set("required", Element::Boolean(BooleanElement::new(value)));
    }

    // -------- Generic field access --------
    pub fn get_field(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn set_field(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        self.object.set(&k, value);
    }

    // -------- Convenience helpers --------
    pub fn has_key(&self, key: &str) -> bool {
        self.object.has_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|member| {
            if let Element::String(s) = &*member.key {
                s.content == key
            } else {
                false
            }
        }) {
            let member = self.object.content.remove(pos);
            Some(*member.value)
        } else {
            None
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.object.content.iter().filter_map(|member| {
            if let Element::String(s) = &*member.key {
                Some(&s.content)
            } else {
                None
            }
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &Element> {
        self.object.content.iter().map(|m| m.value.as_ref())
    }

    pub fn len(&self) -> usize {
        self.object.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.object.content.is_empty()
    }

    // -------- Extension helpers --------
    pub fn get_extension(&self, key: &str) -> Option<&Element> {
        if key.starts_with("x-") {
            self.get_field(key)
        } else {
            None
        }
    }

    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        if k.starts_with("x-") {
            self.set_field(&k, value);
        }
    }

    // -------- Basic validation --------
    /// `content` 字段必填且至少包含一个媒体类型定义
    pub fn validate_basic(&self) -> Result<(), String> {
        let content_obj = self
            .content_prop()
            .ok_or_else(|| "RequestBodyElement.content is required".to_string())?;

        if content_obj.content.is_empty() {
            return Err("RequestBodyElement.content must not be empty".to_string());
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for RequestBodyElement {
    fn from(obj: ObjectElement) -> Self {
        RequestBodyElement::with_content(obj)
    }
}

impl From<RequestBodyElement> for ObjectElement {
    fn from(el: RequestBodyElement) -> Self {
        el.object
    }
}