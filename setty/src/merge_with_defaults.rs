#![cfg(feature = "derive-jsonschema")]

use crate::Value;

/////////////////////////////////////////////////////////////////////////////////////////

pub fn merge_with_defaults(name: &str, value: &mut Value, sch: &Value, defs: &Value) {
    if let Some(r) = sch.get("$ref").and_then(|v| v.as_str()) {
        let (_, tname) = r.rsplit_once('/').unwrap();
        let rsch = &defs[tname];
        return merge_with_defaults(name, value, rsch, defs);
    }
    if let Some(any_of) = sch.get("anyOf").and_then(|v| v.as_array()) {
        // `anyOf` only appears on nullable types
        assert_eq!(any_of.len(), 2);
        assert_eq!(any_of[1]["type"].as_str(), Some("null"));
        let rsch = &any_of[0];
        return merge_with_defaults(name, value, rsch, defs);
    }

    let typ = sch.get("type");
    let typ = if let Some(typ) = typ.and_then(|t| t.as_str()) {
        Some(typ)
    } else if let Some(typ) = typ.and_then(|t| t.as_array()) {
        assert_eq!(typ.len(), 2);
        typ.iter().filter_map(|t| t.as_str()).find(|t| *t != "null")
    } else {
        None
    };

    if typ == Some("object") {
        merge_struct(name, value, sch, defs);
    } else if sch.get("oneOf").is_some() {
        merge_enum(name, value, sch, defs);
    } else if typ == Some("string") && sch.get("enum").is_some() {
        merge_string_enum(name, value, sch, defs);
    } else if typ.is_some() {
        merge_basic_type_wrapper(name, value, sch, defs);
    } else {
        unreachable!("Unsupported schema in `{name}`:\n{sch}")
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn merge_struct(_name: &str, value: &mut Value, sch: &Value, defs: &Value) {
    let value = value.as_object_mut().unwrap();

    let empty = Value::Object(Default::default());
    let props = sch.get("properties").unwrap_or(&empty);

    for (pname, psch) in props.as_object().unwrap() {
        if let Some(pvalue) = value.get_mut(pname) {
            merge_with_defaults(pname, pvalue, psch, defs);
        } else if let Some(default) = psch.get("default") {
            let mut pvalue = default.clone();
            merge_with_defaults(pname, &mut pvalue, psch, defs);
            value.insert(pname.clone(), pvalue);
        } else {
            // Property is not set and does not provide a default
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn merge_enum(name: &str, value: &mut Value, sch: &Value, defs: &Value) {
    let variants = sch.get("oneOf").and_then(|p| p.as_array()).unwrap();

    let tag_property = crate::merge_with_defaults::get_enum_tag_property_name(variants);

    let Some(tag) = value.get(tag_property) else {
        // No tag - can't determine the right variant
        return;
    };

    let Some(vsch) = variants
        .iter()
        .find(|v| v["properties"][tag_property]["const"] == *tag)
    else {
        // Could not find variant corresponding to specified tag
        return;
    };

    merge_with_defaults(name, value, vsch, defs)
}

/////////////////////////////////////////////////////////////////////////////////////////

fn merge_string_enum(_name: &str, _value: &mut Value, _sch: &Value, _defs: &Value) {}

/////////////////////////////////////////////////////////////////////////////////////////

fn merge_basic_type_wrapper(_name: &str, _value: &mut Value, _sch: &Value, _defs: &Value) {}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_enum_tag_property_name(variants: &[Value]) -> &String {
    variants[0].as_object().unwrap()["properties"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_name, prop)| prop.get("type").and_then(|t| t.as_str()) == Some("string"))
        .filter(|(_name, prop)| prop.get("const").and_then(|t| t.as_str()).is_some())
        .map(|(name, __)| name)
        .next()
        .unwrap()
}

/////////////////////////////////////////////////////////////////////////////////////////
