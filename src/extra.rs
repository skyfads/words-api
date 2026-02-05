pub fn normalize_input(term: &str) -> String {
    term.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}
