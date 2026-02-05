/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_deduplicates_explicit_derives() {
    // Always derives Debug, Clone, Eq, deduplicating it with one from Config
    #[setty::derive(setty::Config, Debug, Clone, PartialEq, Eq)]
    struct A {
        x: u32,
    }

    assert_eq!(A { x: 10 }.clone(), A { x: 10 });
}

/////////////////////////////////////////////////////////////////////////////////////////
