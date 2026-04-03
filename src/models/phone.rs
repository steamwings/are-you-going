use phonenumber::country;

/// Normalize raw input to E.164 (e.g. "+15551234567").
/// Tries international parse first, falls back to US.
pub fn normalize_phone(raw: &str) -> Result<String, String> {
    let parsed = phonenumber::parse(None, raw)
        .or_else(|_| phonenumber::parse(Some(country::Id::US), raw))
        .map_err(|e| format!("Invalid phone number: {e}"))?;

    if !phonenumber::is_valid(&parsed) {
        return Err("Invalid phone number".to_string());
    }

    Ok(format!(
        "+{}{}",
        parsed.code().value(),
        parsed.national().value()
    ))
}

/// Parse and reformat for human display (e.g. "+1 (555) 123-4567").
/// Returns None if the input cannot be parsed as a valid number.
pub fn format_phone_display(raw: &str) -> Option<String> {
    let parsed = phonenumber::parse(None, raw)
        .or_else(|_| phonenumber::parse(Some(country::Id::US), raw))
        .ok()?;
    if !phonenumber::is_valid(&parsed) {
        return None;
    }
    let national = phonenumber::format(&parsed)
        .mode(phonenumber::Mode::National)
        .to_string();
    Some(format!("+{} {}", parsed.code().value(), national))
}

#[cfg(test)]
mod tests {
    use super::*;

    // All tests use 202-555-0100 (DC area code, fictional 555-01xx range).

    // --- normalize_phone: input format variations ---

    #[test]
    fn normalize_10_digit() {
        assert_eq!(normalize_phone("2025550100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_with_plus_country_code() {
        assert_eq!(normalize_phone("+12025550100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_11_digit_leading_1() {
        // Users sometimes type the country code without the +
        assert_eq!(normalize_phone("12025550100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_parens_no_space() {
        // (202)555-0100 — no space after closing paren
        assert_eq!(normalize_phone("(202)555-0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_parens_with_space() {
        assert_eq!(normalize_phone("(202) 555-0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_dashes() {
        assert_eq!(normalize_phone("202-555-0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_dots() {
        assert_eq!(normalize_phone("202.555.0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_spaces() {
        assert_eq!(normalize_phone("202 555 0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_dashes_with_country_code() {
        assert_eq!(normalize_phone("1-202-555-0100"), Ok("+12025550100".into()));
    }
    #[test]
    fn normalize_whitespace_padded() {
        // Users copy-paste numbers with surrounding spaces
        assert_eq!(normalize_phone("  2025550100  "), Ok("+12025550100".into()));
    }

    // --- normalize_phone: international numbers ---

    #[test]
    fn normalize_uk_number() {
        // +44 7911 123456 — valid UK mobile
        assert_eq!(normalize_phone("+447911123456"), Ok("+447911123456".into()));
    }

    // --- normalize_phone: invalid inputs ---

    #[test]
    fn normalize_letters_err() {
        assert!(normalize_phone("notaphone").is_err());
    }
    #[test]
    fn normalize_empty_err() {
        assert!(normalize_phone("").is_err());
    }
    #[test]
    fn normalize_too_short_err() {
        assert!(normalize_phone("12345").is_err());
    }
    #[test]
    fn normalize_too_long_err() {
        // 15 digits — exceeds E.164 maximum of 15
        assert!(normalize_phone("123456789012345").is_err());
    }
    #[test]
    fn normalize_only_symbols_err() {
        assert!(normalize_phone("---").is_err());
    }

    // --- format_phone_display: input format variations ---

    #[test]
    fn display_10_digit() {
        assert_eq!(
            format_phone_display("2025550100"),
            Some("+1 (202) 555-0100".into())
        );
    }
    #[test]
    fn display_e164() {
        assert_eq!(
            format_phone_display("+12025550100"),
            Some("+1 (202) 555-0100".into())
        );
    }
    #[test]
    fn display_dashes() {
        assert_eq!(
            format_phone_display("202-555-0100"),
            Some("+1 (202) 555-0100".into())
        );
    }
    #[test]
    fn display_dots() {
        assert_eq!(
            format_phone_display("202.555.0100"),
            Some("+1 (202) 555-0100".into())
        );
    }
    #[test]
    fn display_11_digit_leading_1() {
        assert_eq!(
            format_phone_display("12025550100"),
            Some("+1 (202) 555-0100".into())
        );
    }
    #[test]
    fn display_whitespace_padded() {
        assert_eq!(
            format_phone_display("  2025550100  "),
            Some("+1 (202) 555-0100".into())
        );
    }

    // --- format_phone_display: international numbers ---

    #[test]
    fn display_uk_number() {
        assert_eq!(
            format_phone_display("+447911123456"),
            Some("+44 07911 123456".into())
        );
    }

    // --- format_phone_display: invalid inputs return None ---

    #[test]
    fn display_invalid_letters() {
        assert_eq!(format_phone_display("notaphone"), None);
    }
    #[test]
    fn display_empty() {
        assert_eq!(format_phone_display(""), None);
    }
    #[test]
    fn display_too_short() {
        assert_eq!(format_phone_display("12345"), None);
    }
}
