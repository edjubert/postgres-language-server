#[test]
fn issue_784_repro() {
    let input = "SELECT\n\tt.a\nFROM t\n\nLEFT JOIN u ON u.a = t.a;";
    let result = pgls_statement_splitter::split(input);
    assert_eq!(result.ranges.len(), 1, "got: {:?}", result.ranges.iter().map(|r| &input[*r]).collect::<Vec<_>>());
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[test]
fn issue_784_cte_blank_line() {
    let input = "with a as (select 1)\n\nselect * from a;";
    let result = pgls_statement_splitter::split(input);
    assert_eq!(result.ranges.len(), 1, "got: {:?}", result.ranges.iter().map(|r| &input[*r]).collect::<Vec<_>>());
}

#[test]
fn invalid_falls_back() {
    // incomplete statement (no ;) -> parser fails -> fallback heuristic
    let input = "select 1 from contact\n\nselect 1\n\nselect 3";
    let result = pgls_statement_splitter::split(input);
    assert_eq!(result.ranges.len(), 3, "got: {:?}", result.ranges.iter().map(|r| &input[*r]).collect::<Vec<_>>());
}

#[test]
fn empty_and_whitespace() {
    assert_eq!(pgls_statement_splitter::split("").ranges.len(), 0);
    assert_eq!(pgls_statement_splitter::split("   \n  \n ").ranges.len(), 0);
}

#[test]
fn semicolon_in_comment_and_dollar_quote() {
    let input = "select /*;*/ 1;\nselect $$;$$;";
    let result = pgls_statement_splitter::split(input);
    assert_eq!(result.ranges.len(), 2, "got: {:?}", result.ranges.iter().map(|r| &input[*r]).collect::<Vec<_>>());
}

#[test]
fn legacy_flag_still_splits_on_blank_line() {
    let input = "SELECT\n\tt.a\nFROM t\n\nLEFT JOIN u ON u.a = t.a;";
    let result = pgls_statement_splitter::split_with_options(
        input,
        pgls_statement_splitter::SplitOptions { strategy: pgls_statement_splitter::SplitStrategy::BlankLineHeuristic },
    );
    assert_eq!(result.ranges.len(), 2, "got: {:?}", result.ranges.iter().map(|r| &input[*r]).collect::<Vec<_>>());
}
