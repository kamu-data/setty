#![cfg(feature = "fmt-yaml")]

use std::collections::BTreeMap;

use serde_json::json;

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_combine_strategies() {
    // Always derives Clone, deduplicating it with one from Config
    #[derive(setty::Config)]
    struct Cfg {
        #[config(default, /* combine(replace) */)]
        replace: u32,

        #[config(default, combine(keep))]
        keep: u32,

        #[serde(alias = "string_enum")]
        #[config(default, /* combine(replace) */)]
        string_enu: StringEnum,

        #[cfg(feature = "types-duration-string")]
        #[config(default = std::time::Duration::ZERO, combine(replace))]
        newtype: setty::types::DurationString,

        #[config(default, /* combine(replace) */)]
        vec_replace: Vec<String>,

        #[config(default, combine(merge))]
        vec_merge: Vec<String>,

        #[config(default, /* combine(replace) */)]
        map_replace: BTreeMap<String, String>,

        #[config(default, combine(merge))]
        map_merge: BTreeMap<String, String>,

        #[config(default, combine(replace))]
        obj_replace: Obj,

        #[config(default, /* combine(merge) */)]
        obj_merge: Obj,

        #[serde(rename = "enum_replace")]
        #[config(default, combine(replace))]
        enu_replace: Option<Enu>,

        #[serde(rename = "enum_merge")]
        #[config(default, /* combine(merge) */)]
        enu_merge: Option<Enu>,
    }

    #[derive(setty::Config, setty::Default)]
    enum StringEnum {
        #[default]
        Foo,
        Bar,
        Baz,
    }

    #[derive(setty::Config, setty::Default)]
    struct Obj {
        #[config(default)]
        a: u32,

        #[config(default)]
        b: u32,
    }

    #[derive(setty::Config)]
    #[serde(tag = "type")]
    enum Enu {
        Foo(Foo),
        Bar(Bar),
        Baz(Baz),
    }

    #[derive(setty::Config, setty::Default)]
    struct Foo {
        #[config(default)]
        a: u32,
    }

    #[derive(setty::Config, setty::Default)]
    struct Bar {
        #[config(default)]
        b: u32,
    }

    #[derive(setty::Config, setty::Default)]
    struct Baz {
        #[config(default)]
        a: u32,
        #[config(default)]
        b: u32,
    }

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "replace": 1 }))
        .with_source(json!({ "replace": 2 }))
        .extract()
        .unwrap();

    assert_eq!(cfg.replace, 2);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "keep": 1 }))
        .with_source(json!({ "keep": 2 }))
        .extract()
        .unwrap();

    assert_eq!(cfg.keep, 1);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "string_enum": "Foo" }))
        .with_source(json!({ "string_enum": "Bar" }))
        .extract()
        .unwrap();

    assert_eq!(cfg.string_enu, StringEnum::Bar);

    //
    #[cfg(feature = "types-duration-string")]
    {
        let cfg: Cfg = setty::Config::new()
            .with_source(json!({ "newtype": "10m" }))
            .with_source(json!({ "newtype": "20m" }))
            .extract()
            .unwrap();

        assert_eq!(cfg.newtype, "20m".parse().unwrap());
    }

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "vec_replace": ["a"] }))
        .with_source(json!({ "vec_replace": ["b"] }))
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_replace, ["b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "vec_merge": ["a"]}))
        .with_source(json!({ "vec_merge": ["b"]}))
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_merge, ["a", "b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "vec_merge": ["a"]}))
        // Explicit reset
        .with_source(json!({ "vec_merge": null}))
        .with_source(json!({ "vec_merge": ["b"]}))
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_merge, ["b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({ "map_replace": {"a": "x"}}))
        .with_source(json!({ "map_replace": {"b": "y"}}))
        .extract()
        .unwrap();

    assert_eq!(cfg.map_replace, [("b".to_string(), "y".to_string())].into());

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "map_merge": {
                "a": "x",
                "b": "z",
        }}))
        .with_source(json!({
            "map_merge": {
                "b": "y"
        }}))
        .extract()
        .unwrap();

    assert_eq!(
        cfg.map_merge,
        [
            ("a".to_string(), "x".to_string()),
            ("b".to_string(), "y".to_string())
        ]
        .into()
    );

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "obj_replace": {
                "a": 1,
        }}))
        .with_source(json!({
            "obj_replace": {
                "b": 2,
        }}))
        .extract()
        .unwrap();

    assert_eq!(
        cfg.obj_replace,
        Obj {
            b: 2,
            ..Default::default()
        }
    );

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "obj_merge": {
                "a": 1
        }}))
        .with_source(json!({
            "obj_merge": {
                "b": 2
        }}))
        .extract()
        .unwrap();

    assert_eq!(cfg.obj_merge, Obj { a: 1, b: 2 });

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "enum_replace": {
                "type": "baz",
                "a": 1,
        }}))
        .with_source(json!({
            "enum_replace": {
                "type": "baz",
                "b": 2,
        }}))
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_replace, Some(Enu::Baz(Baz { a: 0, b: 2 })));

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "enum_merge": {
                "type": "baz",
                "a": 1,
        }}))
        .with_source(json!({
            "enum_merge": {
                "type": "baz",
                "b": 2,
        }}))
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_merge, Some(Enu::Baz(Baz { a: 1, b: 2 })));

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "enum_merge": {
                "type": "foo",
                "a": 1,
        }}))
        .with_source(json!({
            "enum_merge": {
                "type": "bar",
                "b": 2,
        }}))
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_merge, Some(Enu::Bar(Bar { b: 2 })));
}

/////////////////////////////////////////////////////////////////////////////////////////
