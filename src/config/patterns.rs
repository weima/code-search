use regex::Regex;

/// Default i18n patterns for various frameworks
pub fn default_patterns() -> Vec<Regex> {
    vec![
        // Ruby patterns
        Regex::new(r#"I18n\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
        Regex::new(r#"\bt\(['"]([^'"]+)['"]\)"#).unwrap(),
        // JavaScript/TypeScript patterns
        Regex::new(r#"i18n\.t\(['"]([^'"]+)['"]\)"#).unwrap(),
        // Vue patterns
        Regex::new(r#"\$t\(['"]([^'"]+)['"]\)"#).unwrap(),
        // React Intl patterns
        Regex::new(r#"id:\s*['"]([^'"]+)['"]"#).unwrap(), // defineMessages
        Regex::new(r#"id=\s*['"]([^'"]+)['"]"#).unwrap(), // FormattedMessage props
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_compile() {
        let patterns = default_patterns();
        assert!(!patterns.is_empty());
        // Updated to reflect added React Intl patterns
        assert_eq!(patterns.len(), 6);
    }

    #[test]
    fn test_ruby_i18n_pattern() {
        let patterns = default_patterns();
        let ruby_pattern = &patterns[0];

        assert!(ruby_pattern.is_match(r#"I18n.t('invoice.labels.add_new')"#));
        assert!(ruby_pattern.is_match(r#"I18n.t("invoice.labels.add_new")"#));
        assert!(!ruby_pattern.is_match(r#"t('invoice.labels.add_new')"#));
    }

    #[test]
    fn test_ruby_t_pattern() {
        let patterns = default_patterns();
        let t_pattern = &patterns[1];

        assert!(t_pattern.is_match(r#"t('invoice.labels.add_new')"#));
        assert!(t_pattern.is_match(r#"t("invoice.labels.add_new")"#));
        // Should match word boundary
        assert!(t_pattern.is_match(r#" t('key')"#));
    }

    #[test]
    fn test_js_i18n_pattern() {
        let patterns = default_patterns();
        let js_pattern = &patterns[2];

        assert!(js_pattern.is_match(r#"i18n.t('invoice.labels.add_new')"#));
        assert!(js_pattern.is_match(r#"i18n.t("invoice.labels.add_new")"#));
    }

    #[test]
    fn test_vue_pattern() {
        let patterns = default_patterns();
        let vue_pattern = &patterns[3];

        assert!(vue_pattern.is_match(r#"$t('invoice.labels.add_new')"#));
        assert!(vue_pattern.is_match(r#"$t("invoice.labels.add_new")"#));
    }
}
