use crate::ast::minim_model::*;

/// OpenAPI `openapi` 字段元素（表示规范版本）
#[derive(Debug, Clone)]
pub struct OpenapiElement {
    pub string: StringElement,
}

impl OpenapiElement {
    pub fn new(content: impl Into<String>) -> Self {
        let content_str = content.into();
        let mut str_elem = StringElement::new(&content_str);
        str_elem.set_element_type("openapi");
        str_elem.add_class("spec-version");
        str_elem.add_class("version");
        Self { string: str_elem }
    }

    pub fn from_element(element: StringElement) -> Self {
        let mut elem = element;
        elem.set_element_type("openapi");
        elem.add_class("spec-version");
        elem.add_class("version");
        Self { string: elem }
    }

    pub fn inner(&self) -> &StringElement {
        &self.string
    }

    pub fn as_str(&self) -> &str {
        self.string.content()
    }
}