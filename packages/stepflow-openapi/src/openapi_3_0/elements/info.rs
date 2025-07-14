use crate::ast::minim_model::*;
use super::contact::ContactElement;
use super::license::LicenseElement;

/// OpenAPI `Info` Element
#[derive(Debug, Clone)]
pub struct InfoElement {
    pub object: ObjectElement,
}

impl InfoElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("info");
        obj.add_class("info");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("info");
        content.add_class("info");
        Self { object: content }
    }

    pub fn title(&self) -> Option<&StringElement> {
        self.object.get("title").and_then(Element::as_string)
    }

    pub fn set_title(&mut self, val: StringElement) {
        self.object.set("title", Element::String(val));
    }

    pub fn description(&self) -> Option<&StringElement> {
        self.object.get("description").and_then(Element::as_string)
    }

    pub fn set_description(&mut self, val: StringElement) {
        self.object.set("description", Element::String(val));
    }

    pub fn terms_of_service(&self) -> Option<&StringElement> {
        self.object.get("termsOfService").and_then(Element::as_string)
    }

    pub fn set_terms_of_service(&mut self, val: StringElement) {
        self.object.set("termsOfService", Element::String(val));
    }

    pub fn contact(&self) -> Option<&ObjectElement> {
        self.object.get("contact").and_then(Element::as_object)
    }

    pub fn set_contact(&mut self, val: ContactElement) {
        self.object.set("contact", Element::Object(val.object));
    }

    pub fn license(&self) -> Option<&ObjectElement> {
        self.object.get("license").and_then(Element::as_object)
    }

    pub fn set_license(&mut self, val: LicenseElement) {
        self.object.set("license", Element::Object(val.object));
    }

    pub fn version(&self) -> Option<&StringElement> {
        self.object.get("version").and_then(Element::as_string)
    }

    pub fn set_version(&mut self, val: StringElement) {
        self.object.set("version", Element::String(val));
    }

    /// 通用读取
    pub fn get(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    /// 通用写入
    pub fn set(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        self.object.set(&k, value);
    }

    // ---------- Convenience helpers ----------

    /// 判断是否包含指定键
    pub fn has_key(&self, key: &str) -> bool {
        self.object.has_key(key)
    }

    /// 移除并返回指定键对应的元素
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

    /// 返回所有键的迭代器
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.object.content.iter().filter_map(|member| {
            if let Element::String(s) = &*member.key {
                Some(&s.content)
            } else {
                None
            }
        })
    }

    /// 返回所有值的迭代器
    pub fn values(&self) -> impl Iterator<Item = &Element> {
        self.object.content.iter().map(|member| member.value.as_ref())
    }

    /// 获取扩展字段（x- 开头）
    pub fn get_extension(&self, key: &str) -> Option<&Element> {
        if key.starts_with("x-") {
            self.get(key)
        } else {
            None
        }
    }

    /// 设置扩展字段（x- 开头）
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        if k.starts_with("x-") {
            self.set(&k, value);
        }
    }

    /// 总成员数量
    pub fn len(&self) -> usize {
        self.object.content.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.object.content.is_empty()
    }

    /// 基础验证：title 与 version 必须存在且非空
    pub fn validate_basic(&self) -> Result<(), String> {
        let title_ok = self.title().map(|t| !t.content.trim().is_empty()).unwrap_or(false);
        let version_ok = self.version().map(|v| !v.content.trim().is_empty()).unwrap_or(false);
        if title_ok && version_ok {
            Ok(())
        } else {
            Err("InfoElement requires non-empty `title` and `version`".to_string())
        }
    }
}

// 与 ObjectElement 互转，便于通用处理
impl From<ObjectElement> for InfoElement {
    fn from(obj: ObjectElement) -> Self {
        InfoElement::with_content(obj)
    }
}

impl From<InfoElement> for ObjectElement {
    fn from(info: InfoElement) -> Self {
        info.object
    }
}