use crate::ast::minim_model::*;
use crate::openapi_3_0::elements::external_documentation::ExternalDocumentationElement;

#[derive(Debug, Clone)]
pub struct SchemaElement {
    pub object: ObjectElement,
}

impl SchemaElement {
    pub fn new() -> Self {
        let mut obj = ObjectElement::new();
        obj.set_element_type("schema");
        Self { object: obj }
    }

    pub fn with_content(content: ObjectElement) -> Self {
        let mut content = content;
        content.set_element_type("schema");
        Self { object: content }
    }

    // ----------- All OpenAPI 3.0 Schema fields -----------

    pub fn title(&self) -> Option<&StringElement> { self.object.get("title").and_then(Element::as_string) }
    pub fn set_title(&mut self, v: StringElement) { self.object.set("title", Element::String(v)); }

    pub fn multiple_of(&self) -> Option<&NumberElement> { self.object.get("multipleOf").and_then(Element::as_number) }
    pub fn set_multiple_of(&mut self, v: NumberElement) { self.object.set("multipleOf", Element::Number(v)); }

    pub fn maximum(&self) -> Option<&NumberElement> { self.object.get("maximum").and_then(Element::as_number) }
    pub fn set_maximum(&mut self, v: NumberElement) { self.object.set("maximum", Element::Number(v)); }

    pub fn exclusive_maximum(&self) -> Option<&BooleanElement> { self.object.get("exclusiveMaximum").and_then(Element::as_boolean) }
    pub fn set_exclusive_maximum(&mut self, v: BooleanElement) { self.object.set("exclusiveMaximum", Element::Boolean(v)); }

    pub fn minimum(&self) -> Option<&NumberElement> { self.object.get("minimum").and_then(Element::as_number) }
    pub fn set_minimum(&mut self, v: NumberElement) { self.object.set("minimum", Element::Number(v)); }

    pub fn exclusive_minimum(&self) -> Option<&BooleanElement> { self.object.get("exclusiveMinimum").and_then(Element::as_boolean) }
    pub fn set_exclusive_minimum(&mut self, v: BooleanElement) { self.object.set("exclusiveMinimum", Element::Boolean(v)); }

    pub fn max_length(&self) -> Option<&NumberElement> { self.object.get("maxLength").and_then(Element::as_number) }
    pub fn set_max_length(&mut self, v: NumberElement) { self.object.set("maxLength", Element::Number(v)); }

    pub fn min_length(&self) -> Option<&NumberElement> { self.object.get("minLength").and_then(Element::as_number) }
    pub fn set_min_length(&mut self, v: NumberElement) { self.object.set("minLength", Element::Number(v)); }

    pub fn pattern(&self) -> Option<&StringElement> { self.object.get("pattern").and_then(Element::as_string) }
    pub fn set_pattern(&mut self, v: StringElement) { self.object.set("pattern", Element::String(v)); }

    pub fn max_items(&self) -> Option<&NumberElement> { self.object.get("maxItems").and_then(Element::as_number) }
    pub fn set_max_items(&mut self, v: NumberElement) { self.object.set("maxItems", Element::Number(v)); }

    pub fn min_items(&self) -> Option<&NumberElement> { self.object.get("minItems").and_then(Element::as_number) }
    pub fn set_min_items(&mut self, v: NumberElement) { self.object.set("minItems", Element::Number(v)); }

    pub fn unique_items(&self) -> Option<&BooleanElement> { self.object.get("uniqueItems").and_then(Element::as_boolean) }
    pub fn set_unique_items(&mut self, v: BooleanElement) { self.object.set("uniqueItems", Element::Boolean(v)); }

    pub fn max_properties(&self) -> Option<&NumberElement> { self.object.get("maxProperties").and_then(Element::as_number) }
    pub fn set_max_properties(&mut self, v: NumberElement) { self.object.set("maxProperties", Element::Number(v)); }

    pub fn min_properties(&self) -> Option<&NumberElement> { self.object.get("minProperties").and_then(Element::as_number) }
    pub fn set_min_properties(&mut self, v: NumberElement) { self.object.set("minProperties", Element::Number(v)); }

    pub fn required(&self) -> Option<&ArrayElement> { self.object.get("required").and_then(Element::as_array) }
    pub fn set_required(&mut self, v: ArrayElement) { self.object.set("required", Element::Array(v)); }

    pub fn enum_values(&self) -> Option<&ArrayElement> { self.object.get("enum").and_then(Element::as_array) }
    pub fn set_enum_values(&mut self, v: ArrayElement) { self.object.set("enum", Element::Array(v)); }

    pub fn schema_type(&self) -> Option<&StringElement> { self.object.get("type").and_then(Element::as_string) }
    pub fn set_schema_type(&mut self, v: StringElement) { self.object.set("type", Element::String(v)); }

    pub fn not(&self) -> Option<&Element> { self.object.get("not") }
    pub fn set_not(&mut self, v: Element) { self.object.set("not", v); }

    pub fn all_of(&self) -> Option<&ArrayElement> { self.object.get("allOf").and_then(Element::as_array) }
    pub fn set_all_of(&mut self, v: ArrayElement) { self.object.set("allOf", Element::Array(v)); }

    pub fn one_of(&self) -> Option<&ArrayElement> { self.object.get("oneOf").and_then(Element::as_array) }
    pub fn set_one_of(&mut self, v: ArrayElement) { self.object.set("oneOf", Element::Array(v)); }

    pub fn any_of(&self) -> Option<&ArrayElement> { self.object.get("anyOf").and_then(Element::as_array) }
    pub fn set_any_of(&mut self, v: ArrayElement) { self.object.set("anyOf", Element::Array(v)); }

    pub fn items(&self) -> Option<&Element> { self.object.get("items") }
    pub fn set_items(&mut self, v: Element) { self.object.set("items", v); }

    pub fn properties(&self) -> Option<&ObjectElement> { self.object.get("properties").and_then(Element::as_object) }
    pub fn set_properties(&mut self, v: ObjectElement) { self.object.set("properties", Element::Object(v)); }

    pub fn additional_properties(&self) -> Option<&Element> { self.object.get("additionalProperties") }
    pub fn set_additional_properties(&mut self, v: Element) { self.object.set("additionalProperties", v); }

    pub fn description(&self) -> Option<&StringElement> { self.object.get("description").and_then(Element::as_string) }
    pub fn set_description(&mut self, v: StringElement) { self.object.set("description", Element::String(v)); }

    pub fn format(&self) -> Option<&StringElement> { self.object.get("format").and_then(Element::as_string) }
    pub fn set_format(&mut self, v: StringElement) { self.object.set("format", Element::String(v)); }

    pub fn default_value(&self) -> Option<&Element> { self.object.get("default") }
    pub fn set_default_value(&mut self, v: Element) { self.object.set("default", v); }

    pub fn nullable(&self) -> Option<&BooleanElement> { self.object.get("nullable").and_then(Element::as_boolean) }
    pub fn set_nullable(&mut self, v: BooleanElement) { self.object.set("nullable", Element::Boolean(v)); }

    pub fn discriminator(&self) -> Option<&ObjectElement> { self.object.get("discriminator").and_then(Element::as_object) }
    pub fn set_discriminator(&mut self, v: ObjectElement) { self.object.set("discriminator", Element::Object(v)); }

    pub fn read_only(&self) -> Option<&BooleanElement> { self.object.get("readOnly").and_then(Element::as_boolean) }
    pub fn set_read_only(&mut self, v: BooleanElement) { self.object.set("readOnly", Element::Boolean(v)); }

    pub fn write_only(&self) -> Option<&BooleanElement> { self.object.get("writeOnly").and_then(Element::as_boolean) }
    pub fn set_write_only(&mut self, v: BooleanElement) { self.object.set("writeOnly", Element::Boolean(v)); }

    pub fn example(&self) -> Option<&Element> { self.object.get("example") }
    pub fn set_example(&mut self, v: Element) { self.object.set("example", v); }

    pub fn external_docs(&self) -> Option<ExternalDocumentationElement> {
        self.object.get("externalDocs").and_then(Element::as_object).map(|obj| ExternalDocumentationElement::with_content(obj.clone()))
    }
    pub fn set_external_docs(&mut self, v: ExternalDocumentationElement) { self.object.set("externalDocs", Element::Object(v.object)); }

    pub fn deprecated(&self) -> Option<&BooleanElement> { self.object.get("deprecated").and_then(Element::as_boolean) }
    pub fn set_deprecated(&mut self, v: BooleanElement) { self.object.set("deprecated", Element::Boolean(v)); }

    pub fn xml(&self) -> Option<&ObjectElement> { self.object.get("xml").and_then(Element::as_object) }
    pub fn set_xml(&mut self, v: ObjectElement) { self.object.set("xml", Element::Object(v)); }

    pub fn ref_(&self) -> Option<&StringElement> { self.object.get("$ref").and_then(Element::as_string) }
    pub fn set_ref(&mut self, v: StringElement) { self.object.set("$ref", Element::String(v)); }

    // ----------- Generic helpers -----------

    pub fn get_field(&self, key: &str) -> Option<&Element> { self.object.get(key) }
    pub fn set_field(&mut self, key: impl Into<String>, value: Element) { let k = key.into(); self.object.set(&k, value); }
    pub fn has_key(&self, key: &str) -> bool { self.object.has_key(key) }
    pub fn remove(&mut self, key: &str) -> Option<Element> {
        if let Some(pos) = self.object.content.iter().position(|m| {
            if let Element::String(s) = &*m.key { s.content == key } else { false }
        }) {
            let m = self.object.content.remove(pos);
            Some(*m.value)
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

    // ----------- Extension helpers -----------

    pub fn get_extension(&self, key: &str) -> Option<&Element> {
        if key.starts_with("x-") { self.get_field(key) } else { None }
    }
    pub fn set_extension(&mut self, key: impl Into<String>, value: Element) {
        let k = key.into();
        if k.starts_with("x-") { self.set_field(&k, value); }
    }

    // ----------- Basic validation -----------

    pub fn validate_basic(&self) -> Result<(), String> {
        // $ref 允许单独存在
        if self.ref_().is_some() { return Ok(()); }
        // 必须有结构性关键字
        let has_structural = self.schema_type().is_some()
            || self.properties().map(|o| !o.content.is_empty()).unwrap_or(false)
            || self.items().is_some()
            || self.all_of().is_some()
            || self.one_of().is_some()
            || self.any_of().is_some()
            || self.not().is_some();
        if has_structural { Ok(()) } else { Err("SchemaElement must have $ref or structural keywords".into()) }
    }
}

// Interop
impl From<ObjectElement> for SchemaElement { fn from(obj: ObjectElement) -> Self { SchemaElement::with_content(obj) } }
impl From<SchemaElement> for ObjectElement { fn from(el: SchemaElement) -> Self { el.object } }