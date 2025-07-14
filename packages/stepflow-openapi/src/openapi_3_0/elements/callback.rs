use crate::ast::minim_model::*;
use serde_json::Value;
/// OpenAPI 3.x CallbackElement
/// 支持运行时表达式检测、元数据操作、$ref 处理等高级功能
#[derive(Debug, Clone)]
pub struct CallbackElement {
    pub object: ObjectElement,
}

impl CallbackElement {
    /// 创建一个空的 CallbackElement，element type 为 "callback"
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("callback");
        Self { object: obj }
    }

    /// 从已有 ObjectElement 创建，并设置 element type
    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("callback");
        Self { object: content }
    }

    /// 获取指定 callback key 的内容（如：post、get 等 operation path）
    pub fn get(&self, key: &str) -> Option<&Element> {
        self.object.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: Element) {
        let key_str = key.into();
        self.object.set(&key_str, value);
    }

    /// 获取整个内容对象
    pub fn content(&self) -> &ObjectElement {
        &self.object
    }

    /// 设置整个内容对象
    pub fn set_content(&mut self, obj: ObjectElement) {
        self.object = obj;
        self.object.set_element_type("callback");
    }

    /// 检查回调是否包含运行时表达式
    pub fn has_runtime_expressions(&self) -> bool {
        self.object.content.iter().any(|member| {
            if let Element::String(key_str) = &*member.key {
                is_runtime_expression(&key_str.content)
            } else {
                false
            }
        })
    }

    /// 获取所有运行时表达式的键
    pub fn get_runtime_expression_keys(&self) -> Vec<String> {
        self.object.content.iter()
            .filter_map(|member| {
                if let Element::String(key_str) = &*member.key {
                    if is_runtime_expression(&key_str.content) {
                        Some(key_str.content.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// 过滤出包含运行时表达式的元素
    pub fn filter_runtime_expressions(&self) -> Vec<(&String, &Element)> {
        self.object.content.iter()
            .filter_map(|member| {
                if let Element::String(key_str) = &*member.key {
                    if is_runtime_expression(&key_str.content) {
                        Some((&key_str.content, member.value.as_ref()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// 检查回调是否包含 $ref 引用
    pub fn has_references(&self) -> bool {
        self.object.content.iter().any(|member| {
            if let Element::Object(obj) = member.value.as_ref() {
                obj.has_key("$ref")
            } else {
                false
            }
        })
    }

    /// 获取所有 $ref 引用的路径
    pub fn get_reference_paths(&self) -> Vec<String> {
        self.object.content.iter()
            .filter_map(|member| {
                if let Element::Object(obj) = member.value.as_ref() {
                    if let Some(Element::String(ref_str)) = obj.get("$ref") {
                        Some(ref_str.content.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// 为指定键的元素设置元数据
    pub fn set_meta_property(&mut self, key: &str, meta_key: &str, meta_value: Value) -> bool {
        for member in &mut self.object.content {
            if let Element::String(key_str) = &*member.key {
                if key_str.content == key {
                    if let Element::Object(ref mut obj) = *member.value {
                        obj.meta.properties.insert(meta_key.to_string(), meta_value);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取指定键的元素的元数据
    pub fn get_meta_property(&self, key: &str, meta_key: &str) -> Option<&Value> {
        for member in &self.object.content {
            if let Element::String(key_str) = &*member.key {
                if key_str.content == key {
                    if let Element::Object(obj) = member.value.as_ref() {
                        return obj.meta.properties.get(meta_key);
                    }
                }
            }
        }
        None
    }

    /// 为所有 PathItem 元素设置运行时表达式元数据
    pub fn decorate_path_items_with_expressions(&mut self) {
        let mut updates = Vec::new();
        
        for member in &self.object.content {
            if let Element::String(key_str) = &*member.key {
                let key = &key_str.content;
                if is_runtime_expression(key) {
                    if let Element::Object(obj) = member.value.as_ref() {
                        if obj.element == "pathItem" || contains_path_item_operations(member.value.as_ref()) {
                            updates.push((key.clone(), key.clone()));
                        }
                    }
                }
            }
        }
        
        for (key, expression) in updates {
            self.set_meta_property(&key, "runtime-expression", Value::String(expression));
        }
    }

    /// 获取所有 PathItem 元素
    pub fn get_path_items(&self) -> Vec<(&String, &ObjectElement)> {
        self.object.content.iter()
            .filter_map(|member| {
                if let Element::String(key_str) = &*member.key {
                    if let Element::Object(obj) = member.value.as_ref() {
                        if obj.element == "pathItem" || contains_path_item_operations(member.value.as_ref()) {
                            Some((&key_str.content, obj))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// 检查指定键的元素是否为 PathItem
    pub fn is_path_item(&self, key: &str) -> bool {
        match self.get(key) {
            Some(element @ Element::Object(obj)) => {
                obj.element == "pathItem" || contains_path_item_operations(element)
            }
            _ => false,
        }
    }

    /// 获取指定键的 PathItem 元素的运行时表达式
    pub fn get_path_item_expression(&self, key: &str) -> Option<String> {
        self.get_meta_property(key, "runtime-expression")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// 迭代所有成员
    pub fn iter(&self) -> impl Iterator<Item = (&Element, &Element)> {
        self.object.content.iter().map(|member| (member.key.as_ref(), member.value.as_ref()))
    }

    /// 计算回调中的表达式数量
    pub fn expression_count(&self) -> usize {
        self.get_runtime_expression_keys().len()
    }

    /// 检查回调是否为空
    pub fn is_empty(&self) -> bool {
        self.object.content.is_empty()
    }

    /// 获取回调的键数量
    pub fn len(&self) -> usize {
        self.object.content.len()
    }

    /// 判断是否包含指定键
    pub fn has_key(&self, key: &str) -> bool {
        self.object.has_key(key)
    }

    /// 移除并返回指定键对应的元素
    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|member| {
            if let Element::String(string_el) = &*member.key {
                string_el.content == key
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

    /// 以迭代器形式返回所有键
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.object.content.iter().filter_map(|member| {
            if let Element::String(key_str) = &*member.key {
                Some(&key_str.content)
            } else {
                None
            }
        })
    }

    /// 以迭代器形式返回所有值
    pub fn values(&self) -> impl Iterator<Item = &Element> {
        self.object.content.iter().map(|member| member.value.as_ref())
    }
}

/// 检测字符串是否为运行时表达式格式 {expression}
fn is_runtime_expression(key: &str) -> bool {
    key.starts_with('{') && key.ends_with('}') && key.len() > 2 && key.len() <= 2085
}

/// 检测元素是否包含 PathItem 操作（GET, POST, PUT, DELETE 等）
fn contains_path_item_operations(element: &Element) -> bool {
    if let Element::Object(obj) = element {
        let operations = ["get", "post", "put", "delete", "options", "head", "patch", "trace"];
        operations.iter().any(|op| obj.has_key(op))
    } else {
        false
    }
}

// 提供与 ObjectElement 的互转，便于通用处理
impl From<ObjectElement> for CallbackElement {
    fn from(obj: ObjectElement) -> Self {
        CallbackElement::with_content(obj)
    }
}

impl From<CallbackElement> for ObjectElement {
    fn from(cb: CallbackElement) -> Self {
        cb.object
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_element_new() {
        let callback = CallbackElement::new();
        assert_eq!(callback.object.element, "callback");
        assert!(callback.object.content.is_empty());
    }

    #[test]
    fn test_callback_element_with_content() {
        let mut obj = ObjectElement::new();
        obj.set("test", Element::String(StringElement::new("value")));
        
        let callback = CallbackElement::with_content(obj);
        assert_eq!(callback.object.element, "callback");
        assert!(callback.get("test").is_some());
    }

    #[test]
    fn test_callback_element_get_set() {
        let mut callback = CallbackElement::new();
        
        // 测试设置和获取
        let test_value = Element::String(StringElement::new("test_value"));
        callback.set("test_key", test_value);
        
        let retrieved = callback.get("test_key");
        assert!(retrieved.is_some());
        
        if let Some(Element::String(s)) = retrieved {
            assert_eq!(s.content, "test_value");
        } else {
            panic!("Expected string element");
        }
    }

    #[test]
    fn test_runtime_expressions() {
        let mut callback = CallbackElement::new();
        
        // 添加运行时表达式
        let mut path_item = ObjectElement::new();
        path_item.set_element_type("pathItem");
        path_item.set("post", Element::Object(ObjectElement::new()));
        callback.set("{$request.body#/callbackUrl}", Element::Object(path_item));
        
        // 添加普通键
        callback.set("normalKey", Element::String(StringElement::new("value")));
        
        assert!(callback.has_runtime_expressions());
        assert_eq!(callback.get_runtime_expression_keys().len(), 1);
        assert_eq!(callback.get_runtime_expression_keys()[0], "{$request.body#/callbackUrl}");
        
        let expressions = callback.filter_runtime_expressions();
        assert_eq!(expressions.len(), 1);
        assert_eq!(expressions[0].0, "{$request.body#/callbackUrl}");
    }

    #[test]
    fn test_references() {
        let mut callback = CallbackElement::new();
        
        // 添加引用
        let mut ref_obj = ObjectElement::new();
        ref_obj.set("$ref", Element::String(StringElement::new("#/components/pathItems/webhook")));
        callback.set("webhookRef", Element::Object(ref_obj));
        
        // 添加普通对象
        callback.set("normalObj", Element::Object(ObjectElement::new()));
        
        assert!(callback.has_references());
        let ref_paths = callback.get_reference_paths();
        assert_eq!(ref_paths.len(), 1);
        assert_eq!(ref_paths[0], "#/components/pathItems/webhook");
    }

    #[test]
    fn test_meta_property_operations() {
        let mut callback = CallbackElement::new();
        
        let mut path_item = ObjectElement::new();
        path_item.set_element_type("pathItem");
        callback.set("testPath", Element::Object(path_item));
        
        // 设置元数据
        let success = callback.set_meta_property("testPath", "runtime-expression", Value::String("{test}".to_string()));
        assert!(success);
        
        // 获取元数据
        let meta_value = callback.get_meta_property("testPath", "runtime-expression");
        assert!(meta_value.is_some());
        if let Some(Value::String(expr)) = meta_value {
            assert_eq!(expr, "{test}");
        }
    }

    #[test]
    fn test_decorate_path_items_with_expressions() {
        let mut callback = CallbackElement::new();
        
        let mut path_item = ObjectElement::new();
        path_item.set_element_type("pathItem");
        path_item.set("get", Element::Object(ObjectElement::new()));
        callback.set("{$request.body#/url}", Element::Object(path_item));
        
        callback.decorate_path_items_with_expressions();
        
        let expression = callback.get_path_item_expression("{$request.body#/url}");
        assert!(expression.is_some());
        assert_eq!(expression.unwrap(), "{$request.body#/url}");
    }

    #[test]
    fn test_path_item_operations() {
        let mut callback = CallbackElement::new();
        
        // 添加 PathItem
        let mut path_item = ObjectElement::new();
        path_item.set_element_type("pathItem");
        path_item.set("post", Element::Object(ObjectElement::new()));
        callback.set("pathItem1", Element::Object(path_item));
        
        // 添加含有操作的对象（未设置 pathItem 类型）
        let mut operations_obj = ObjectElement::new();
        operations_obj.set("get", Element::Object(ObjectElement::new()));
        callback.set("pathItem2", Element::Object(operations_obj));
        
        // 添加普通对象
        callback.set("normalObj", Element::Object(ObjectElement::new()));
        
        assert!(callback.is_path_item("pathItem1"));
        assert!(callback.is_path_item("pathItem2"));
        assert!(!callback.is_path_item("normalObj"));
        
        let path_items = callback.get_path_items();
        assert_eq!(path_items.len(), 2);
    }

    #[test]
    fn test_callback_iteration_and_stats() {
        let mut callback = CallbackElement::new();
        
        callback.set("{expr1}", Element::String(StringElement::new("value1")));
        callback.set("{expr2}", Element::String(StringElement::new("value2")));
        callback.set("normal", Element::String(StringElement::new("value3")));
        
        assert_eq!(callback.len(), 3);
        assert!(!callback.is_empty());
        assert_eq!(callback.expression_count(), 2);
        
        let mut count = 0;
        for (key, value) in callback.iter() {
            assert!(matches!(key, Element::String(_)) || matches!(key, Element::Object(_)));
            assert!(matches!(value, Element::String(_)));
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_complex_callback_with_mixed_content() {
        let mut callback = CallbackElement::new();
        
        // 运行时表达式 + PathItem
        let mut path_item1 = ObjectElement::new();
        path_item1.set_element_type("pathItem");
        path_item1.set("post", Element::Object(ObjectElement::new()));
        callback.set("{$request.body#/webhook}", Element::Object(path_item1));
        
        // 普通键 + 引用
        let mut ref_obj = ObjectElement::new();
        ref_obj.set("$ref", Element::String(StringElement::new("#/components/callbacks/myCallback")));
        callback.set("callbackRef", Element::Object(ref_obj));
        
        // 普通键 + 普通对象
        let mut normal_obj = ObjectElement::new();
        normal_obj.set("description", Element::String(StringElement::new("A normal callback")));
        callback.set("normalCallback", Element::Object(normal_obj));
        
        // 验证各种功能
        assert!(callback.has_runtime_expressions());
        assert!(callback.has_references());
        assert_eq!(callback.len(), 3);
        assert_eq!(callback.expression_count(), 1);
        assert_eq!(callback.get_runtime_expression_keys().len(), 1);
        assert_eq!(callback.get_reference_paths().len(), 1);
        assert_eq!(callback.get_path_items().len(), 1);
        
        // 装饰 PathItem
        callback.decorate_path_items_with_expressions();
        let expr = callback.get_path_item_expression("{$request.body#/webhook}");
        assert!(expr.is_some());
        assert_eq!(expr.unwrap(), "{$request.body#/webhook}");
    }
}