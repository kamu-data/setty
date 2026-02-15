#![cfg(feature = "derive-jsonschema")]

use crate::Value;

/////////////////////////////////////////////////////////////////////////////////////////

pub type OnDeprecatedClb =
    dyn Fn(&[&str], /* reason */ Option<&str>, /* since */ Option<&str>) + 'static;

/////////////////////////////////////////////////////////////////////////////////////////

pub fn default_deprecation_clb(path: &[&str], reason: Option<&str>, since: Option<&str>) {
    use std::fmt::Write;

    let mut s = String::new();

    write!(
        &mut s,
        "WARNING: Config property `{}` is deprecated",
        path.join(".")
    )
    .unwrap();

    if let Some(since) = since {
        write!(&mut s, " since version: {since}").unwrap();
    }
    if let Some(reason) = reason {
        s.push('\n');
        write!(&mut s, "  {}", reason.replace("\n", "\n  ")).unwrap();
    }

    eprintln!("{s}");
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Helper that scans a value for deprecated fields using the schema.
pub fn check_deprecated_fields(schema: &Value, value: &Value, clb: &OnDeprecatedClb) {
    let defs = &schema["$defs"];
    let mut path = Vec::new();
    check_deprecated_fields_rec(&mut path, schema, value, defs, clb)
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Helper that scans a value for deprecated fields using the schema.
fn check_deprecated_fields_rec<'a>(
    path: &mut Vec<&'a str>,
    sch: &'a Value,
    value: &'a Value,
    defs: &'a Value,
    clb: &OnDeprecatedClb,
) {
    if let Some(r) = sch.get("$ref").and_then(|v| v.as_str()) {
        let (_, tname) = r.rsplit_once('/').unwrap();
        let rsch = &defs[tname];
        return check_deprecated_fields_rec(path, rsch, value, defs, clb);
    }
    if let Some(any_of) = sch.get("anyOf").and_then(|v| v.as_array()) {
        // `anyOf` only appears on nullable types
        assert_eq!(any_of.len(), 2);
        assert_eq!(any_of[1]["type"].as_str(), Some("null"));
        let rsch = &any_of[0];
        return check_deprecated_fields_rec(path, rsch, value, defs, clb);
    }
    if let Some(one_of) = sch.get("oneOf").and_then(|v| v.as_array()) {
        // TODO: A perfectionist should consider the tag here
        for vsch in one_of {
            check_deprecated_fields_rec(path, vsch, value, defs, clb);
        }
    }

    let Some(properties) = sch.get("properties").and_then(|v| v.as_object()) else {
        return;
    };

    let Some(value) = value.as_object() else {
        return;
    };

    for (pname, pvalue) in value {
        path.push(pname.as_str());

        let Some(pschema) = properties.get(pname) else {
            continue;
        };

        if pschema.get("deprecated") == Some(&Value::Bool(true)) {
            let reason = pschema["deprecation"]["reason"].as_str();
            let since = pschema["deprecation"]["since"].as_str();
            clb(path, reason, since);
        }

        check_deprecated_fields_rec(path, pschema, pvalue, defs, clb);
        path.pop();
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
