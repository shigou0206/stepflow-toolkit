use crate::ast::minim_model::*;

#[derive(Debug, Clone)]
pub struct MediaTypeElement {
    pub object: ObjectElement,
}

impl MediaTypeElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("mediaType");
        Self { object: obj }
    }

    pub fn with_content(mut content: ObjectElement) -> Self {
        content.set_element_type("mediaType");
        Self { object: content }
    }

    // schema: SchemaElement or ReferenceElement (ObjectElement)
    pub fn schema(&self) -> Option<&Element> {
        self.object.get("schema")
    }

    pub fn set_schema(&mut self, value: Element) {
        self.object.set("schema", value);
    }

    // example: any Element
    pub fn example(&self) -> Option<&Element> {
        self.object.get("example")
    }

    pub fn set_example(&mut self, value: Element) {
        self.object.set("example", value);
    }

    // examples: ObjectElement
    pub fn examples(&self) -> Option<&ObjectElement> {
        self.object.get("examples").and_then(Element::as_object)
    }

    pub fn set_examples(&mut self, value: ObjectElement) {
        self.object.set("examples", Element::Object(value));
    }

    // encoding: ObjectElement
    pub fn encoding(&self) -> Option<&ObjectElement> {
        self.object.get("encoding").and_then(Element::as_object)
    }

    pub fn set_encoding(&mut self, value: ObjectElement) {
        self.object.set("encoding", Element::Object(value));
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
    /// 至少包含 schema、example、examples、encoding 中的一个字段
    pub fn validate_basic(&self) -> Result<(), String> {
        if self.schema().is_none()
            && self.example().is_none()
            && self.examples().map(|o| o.content.is_empty()).unwrap_or(true)
            && self.encoding().map(|o| o.content.is_empty()).unwrap_or(true)
        {
            return Err("MediaTypeElement must have at least one of: schema, example, examples, encoding".to_string());
        }
        Ok(())
    }
}

// Interop with ObjectElement
impl From<ObjectElement> for MediaTypeElement {
    fn from(obj: ObjectElement) -> Self {
        MediaTypeElement::with_content(obj)
    }
}

impl From<MediaTypeElement> for ObjectElement {
    fn from(el: MediaTypeElement) -> Self {
        el.object
    }
}