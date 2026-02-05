#![cfg(feature = "fmt-yaml")]

/////////////////////////////////////////////////////////////////////////////////////////

use std::collections::BTreeMap;

use setty::format::Yaml;

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

        #[cfg(feature = "type-duration-string05")]
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
        .with_source_str::<Yaml>(
            r#"
            replace: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            replace: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.replace, 2);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            keep: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            keep: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.keep, 1);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            string_enum: Foo
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            string_enum: Bar
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.string_enu, StringEnum::Bar);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            newtype: 10m
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            newtype: 20m
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.newtype, "20m".parse().unwrap());

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            vec_replace:
                - a
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            vec_replace:
                - b
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_replace, ["b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            vec_merge:
                - a
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            vec_merge:
                - b
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_merge, ["a", "b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            vec_merge:
                - a
            "#,
        )
        // Explicit reset
        .with_source_str::<Yaml>(
            r#"
            vec_merge: null
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            vec_merge:
                - b
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.vec_merge, ["b"]);

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            map_replace:
                a: x
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            map_replace:
                b: y
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.map_replace, [("b".to_string(), "y".to_string())].into());

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            map_merge:
                a: x
                b: z
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            map_merge:
                b: y
            "#,
        )
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
        .with_source_str::<Yaml>(
            r#"
            obj_replace:
                a: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            obj_replace:
                b: 2
            "#,
        )
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
        .with_source_str::<Yaml>(
            r#"
            obj_merge:
                a: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            obj_merge:
                b: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.obj_merge, Obj { a: 1, b: 2 });

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            enum_replace:
                type: baz
                a: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            enum_replace:
                type: baz
                b: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_replace, Some(Enu::Baz(Baz { a: 0, b: 2 })));

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            enum_merge:
                type: baz
                a: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            enum_merge:
                type: baz
                b: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_merge, Some(Enu::Baz(Baz { a: 1, b: 2 })));

    //
    let cfg: Cfg = setty::Config::new()
        .with_source_str::<Yaml>(
            r#"
            enum_merge:
                type: foo
                a: 1
            "#,
        )
        .with_source_str::<Yaml>(
            r#"
            enum_merge:
                type: bar
                b: 2
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg.enu_merge, Some(Enu::Bar(Bar { b: 2 })));
}

/////////////////////////////////////////////////////////////////////////////////////////
