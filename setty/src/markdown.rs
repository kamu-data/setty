#![cfg(feature = "derive-jsonschema")]

use crate::Value;
use std::fmt::Write;

/////////////////////////////////////////////////////////////////////////////////////////

pub fn schema_to_markdown(schema: &schemars::Schema) -> String {
    use std::fmt::Write;

    let mut ret = String::new();
    let buf = &mut ret;
    let schema = schema.as_value();

    write_type(buf, schema["title"].as_str().unwrap(), schema).unwrap();

    for (name, sch) in schema["$defs"].as_object().unwrap() {
        writeln!(buf).unwrap();
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

    writeln!(buf, "| Field | Type | Required | Description |")?;
    writeln!(buf, "|---|---|---|---|")?;

    let mut required = std::collections::BTreeSet::new();
    if let Some(req) = schema.get("required").and_then(|v| v.as_array()) {
        for v in req {
            required.insert(v.as_str().unwrap());
        }
    }

    for (pname, psch) in properties {
        let mut is_required = required.contains(pname.as_str());

        let ptype = if let Some(ty) = psch.get("type") {
            if let Some(ty) = ty.as_str() {
                format!("`{ty}`")
            } else if let Some(ty) = ty.as_array()
                && ty.len() == 2
            {
                let ty = ty
                    .iter()
                    .filter_map(|t| t.as_str())
                    .find(|t| *t != "null")
                    .unwrap();

                is_required = false;
                format!("`{ty}`")
            } else {
                unreachable!("Unsupported schema in {name}.{pname}:\n{psch}")
            }
        } else if let Some(r) = psch.get("$ref").and_then(|v| v.as_str()) {
            let (_, tname) = r.rsplit_once('/').unwrap();
            format!("[`{tname}`](#{})", name_to_id(tname))
        } else if let Some(any_of) = psch.get("anyOf").and_then(|v| v.as_array()) {
            assert_eq!(any_of.len(), 2);
            assert_eq!(any_of[1]["type"].as_str(), Some("null"));
            let r = any_of[0]["$ref"].as_str().unwrap();
            let (_, tname) = r.rsplit_once('/').unwrap();
            format!("[`{tname}`](#{})", name_to_id(tname))
        } else {
            unreachable!("Unsupported schema in {name}.{pname}:\n{psch}")
        };

        let pdesc = escape_str_for_table(
            psch.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );

        let req = if is_required { "`V`" } else { "" };

        writeln!(buf, "| `{pname}` | {ptype} | {req} | {pdesc} |")?;
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////

fn write_enum(buf: &mut String, name: &str, schema: &Value) -> Result<(), std::fmt::Error> {
    let Some(variants) = schema.get("oneOf").and_then(|p| p.as_array()) else {
        unreachable!()
    };

    let tag_property = crate::merge_with_defaults::get_enum_tag_property_name(variants);

    writeln!(buf, "| Variants |")?;
    writeln!(buf, "|---|")?;

    for variant in variants.iter() {
        let tag = variant["properties"][tag_property]["const"]
            .as_str()
            .unwrap();

        let vname = format!("{name}::{tag}");

        writeln!(buf, "| [`{tag}`](#{}) |", name_to_id(&vname))?;
    }

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
    writeln!(buf, "| Variants |")?;
    writeln!(buf, "|---|")?;

    for variant in schema.get("enum").and_then(|v| v.as_array()).unwrap() {
        writeln!(buf, "| `{}` |", variant.as_str().unwrap())?;
    }

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

// HACKS: TODO: Proper Markdown parser?
fn escape_str_for_table(s: &str) -> String {
    s.replace("```sh", "")
        .replace("```", "")
        .replace("|", "\\|")
        .replace("\n", "<br>")
}

/////////////////////////////////////////////////////////////////////////////////////////
