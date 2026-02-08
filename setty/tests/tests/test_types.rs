#![cfg(feature = "fmt-yaml")]

use serde_json::json;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "types-chrono")]
#[test]
fn test_rfc3339() {
    #[derive(setty::Config)]
    struct Cfg {
        dt: setty::types::DateTime,
    }

    //
    let cfg: Cfg = setty::Config::new()
        .with_source(json!({
            "dt": "1996-12-19T16:39:57-08:00",
        }))
        .extract()
        .unwrap();

    assert_eq!(
        *cfg.dt.as_chrono(),
        chrono::DateTime::<chrono::FixedOffset>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(1996, 12, 20).unwrap(),
                chrono::NaiveTime::from_hms_opt(0, 39, 57).unwrap()
            ),
            chrono::FixedOffset::west_opt(8 * 3600).unwrap(),
        )
    );

    pretty_assertions::assert_eq!(
        serde_json::to_value(&cfg).unwrap(),
        serde_json::json!({"dt": "1996-12-19T16:39:57-08:00"}),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
