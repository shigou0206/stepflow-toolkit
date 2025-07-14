use crate::ast::minim_model::*;

/// OpenAPI SecurityRequirement Element
#[derive(Debug, Clone)]
pub struct SecurityRequirementElement {
    pub object: ObjectElement,
}

impl SecurityRequirementElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("securityRequirement");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("securityRequirement");
        Self { object: content }
    }

    /// 获取某个 security scheme 的 scopes 列表
    pub fn get_scopes(&self, scheme_name: &str) -> Option<&ArrayElement> {
        self.object
            .get(scheme_name)
            .and_then(Element::as_array)
    }

    /// 设置某个 security scheme 的 scopes 列表
    pub fn set_scopes(&mut self, scheme_name: &str, scopes: ArrayElement) {
        self.object
            .set(scheme_name, Element::Array(scopes));
    }

    /// 列出所有 security scheme 名称
    pub fn scheme_names(&self) -> Vec<String> {
        self.object
            .content
            .iter()
            .filter_map(|member| {
                if let Element::String(string_elem) = &*member.key {
                    Some(string_elem.content.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}