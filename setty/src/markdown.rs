#![cfg(feature = "derive-jsonschema")]

use crate::Value;
use std::fmt::Write;

/////////////////////////////////////////////////////////////////////////////////////////

/// Convert a [`schemars::Schema`] into a Markdown string describing types,
/// fields and variants. This is intended for inclusion in user-facing
/// documentation and CLI help.
pub fn schema_to_markdown(schema: &schemars::Schema) -> String {
    use std::fmt::Write;

    let mut ret = String::new();
    let buf = &mut ret;
    let schema = schema.as_value();

    write_type(buf, schema["title"].as_str().unwrap(), schema).unwrap();

    let empty = serde_json::Map::default();
    for (name, sch) in schema["$defs"].as_object().unwrap_or(&empty) {
        writeln!(buf).unwrap();
        write_type(buf, name, sch).unwrap();
    }

    ret
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_type(buf: &mut String, name: &str, schema: &Value) -> Result<(), std::fmt::Error> {
    writeln!(buf, "## `{name}`")?;
    writeln!(buf)?;

    if let Some(desc) = schema.get("description").and_then(|v| v.as_str()) {
        writeln!(buf, "{desc}")?;
        writeln!(buf)?;
    }

    let typ = schema.get("type").and_then(|p| p.as_str());

    if typ == Some("object") {
        write_struct(buf, name, schema)?;
    } else if schema.get("oneOf").is_some() {
        write_enum(buf, name, schema)?;
    } else if typ == Some("string") {
        if schema.get("enum").is_some() {
            write_string_enum(buf, name, schema)?;
        } else {
            write_basic_type_wrapper(buf, name, schema)?;
        }
    } else {
        unreachable!("Unsupported schema in {name}:\n{schema}")
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_struct(buf: &mut String, name: &str, schema: &Value) -> Result<(), std::fmt::Error> {
    let empty = serde_json::Map::default();
    let properties = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .unwrap_or(&empty);

    writeln!(buf, "<table>")?;
    writeln!(
        buf,
        "<thead><tr><th>Field</th><th>Type</th><th>Default</th><th>Description</th></tr></thead>"
    )?;
    writeln!(buf, "<tbody>")?;

    let mut required = std::collections::BTreeSet::new();
    if let Some(req) = schema.get("required").and_then(|v| v.as_array()) {
        for v in req {
            required.insert(v.as_str().unwrap());
        }
    }

    for (pname, psch) in properties {
        let mut is_required = required.contains(pname.as_str());

        writeln!(buf, "<tr>")?;

        // Name
        writeln!(buf, "<td><code>{pname}</code></td>")?;

        // Type
        if let Some(ty) = psch.get("type") {
            if let Some(ty) = ty.as_str() {
                writeln!(buf, "<td><code>{ty}</code></td>")?;
            } else if let Some(ty) = ty.as_array()
                && ty.len() == 2
            {
                let ty = ty
                    .iter()
                    .filter_map(|t| t.as_str())
                    .find(|t| *t != "null")
                    .unwrap();

                is_required = false;
                writeln!(buf, "<td><code>{ty}</code></td>")?;
            } else {
                unreachable!("Unsupported schema in {name}.{pname}:\n{psch}")
            }
        } else if let Some(r) = psch.get("$ref").and_then(|v| v.as_str()) {
            let (_, tname) = r.rsplit_once('/').unwrap();
            writeln!(
                buf,
                "<td><a href=\"#{}\"><code>{tname}</code></a></td>",
                name_to_id(tname)
            )?;
        } else if let Some(any_of) = psch.get("anyOf").and_then(|v| v.as_array()) {
            assert_eq!(any_of.len(), 2);
            assert_eq!(any_of[1]["type"].as_str(), Some("null"));
            let r = any_of[0]["$ref"].as_str().unwrap();
            let (_, tname) = r.rsplit_once('/').unwrap();
            writeln!(
                buf,
                "<td><a href=\"#{}\"><code>{tname}</code></a></td>",
                name_to_id(tname)
            )?;
        } else {
            unreachable!("Unsupported schema in {name}.{pname}:\n{psch}")
        };

        // Default
        let null = serde_json::Value::Null;
        if is_required {
            writeln!(buf, "<td></td>")?;
        } else {
            let default = psch.get("default").unwrap_or(&null);
            let default_str = serde_json::to_string_pretty(default).unwrap();
            let escaped = html_escape::encode_safe(&default_str);

            if escaped.contains('\n') {
                writeln!(
                    buf,
                    "<td><pre><code class=\"language-json\">{escaped}</code></pre></td>"
                )?;
            } else {
                writeln!(
                    buf,
                    "<td><code class=\"language-json\">{escaped}</code></td>"
                )?;
            }
        };

        // Description
        let pdesc = psch
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if pdesc.contains('\n') || pdesc.contains('`') {
            writeln!(buf, "<td>\n\n{pdesc}\n\n</td>")?;
        } else {
            let pdesc = html_escape::encode_script(pdesc);
            writeln!(buf, "<td>{pdesc}</td>")?;
        }

        writeln!(buf, "</tr>")?;
    }

    writeln!(buf, "</tbody>")?;
    writeln!(buf, "</table>")?;

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_enum(buf: &mut String, name: &str, schema: &Value) -> Result<(), std::fmt::Error> {
    let Some(variants) = schema.get("oneOf").and_then(|p| p.as_array()) else {
        unreachable!()
    };

    let tag_property = crate::merge_with_defaults::get_enum_tag_property_name(variants);

    writeln!(buf, "<table>")?;
    writeln!(buf, "<thead><tr><th>Variants</th></tr></thead>")?;
    writeln!(buf, "<tbody>")?;

    for variant in variants.iter() {
        let tag = variant["properties"][tag_property]["const"]
            .as_str()
            .unwrap();

        let vname = format!("{name}::{tag}");

        writeln!(
            buf,
            "<tr><td><a href=\"#{}\"><code>{tag}</code></a></td></tr>",
            name_to_id(&vname)
        )?;
    }

    writeln!(buf, "</tbody>")?;
    writeln!(buf, "</table>")?;

    for vsch in variants.iter() {
        writeln!(buf)?;
        writeln!(buf)?;

        let tag = vsch["properties"][tag_property]["const"].as_str().unwrap();
        let vname = format!("{name}::{tag}");
        write_type(buf, &vname, vsch)?;
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_string_enum(buf: &mut String, _name: &str, schema: &Value) -> Result<(), std::fmt::Error> {
    writeln!(buf, "<table>")?;
    writeln!(buf, "<thead><tr><th>Variants</th></tr></thead>")?;
    writeln!(buf, "<tbody>")?;

    for variant in schema.get("enum").and_then(|v| v.as_array()).unwrap() {
        let tag = variant.as_str().unwrap();
        writeln!(
            buf,
            "<tr><td><code>{}</code></td></tr>",
            html_escape::encode_safe(tag)
        )?;
    }

    writeln!(buf, "</tbody>")?;
    writeln!(buf, "</table>")?;

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_basic_type_wrapper(
    buf: &mut String,
    _name: &str,
    schema: &Value,
) -> Result<(), std::fmt::Error> {
    let typ = schema.get("type").and_then(|p| p.as_str()).unwrap();
    writeln!(buf, "Base type: `{typ}`")?;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn name_to_id(name: &str) -> String {
    name.replace(":", "").to_lowercase()
}

/////////////////////////////////////////////////////////////////////////////////////////
