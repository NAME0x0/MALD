/// Simple fuzzy matching score.
/// Returns a score where higher is better. Returns 0 if no match.
pub fn fuzzy_score(query: &str, target: &str) -> u32 {
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    if query_lower.is_empty() {
        return 1; // empty query matches everything
    }

    // Exact substring match → high score
    if target_lower.contains(&query_lower) {
        return 100 + (1000 / (target.len() as u32 + 1));
    }

    // Subsequence match
    let mut qi = query_lower.chars().peekable();
    let mut score: u32 = 0;
    let mut consecutive = 0u32;
    let mut prev_matched = false;

    for (i, tc) in target_lower.chars().enumerate() {
        if let Some(&qc) = qi.peek() {
            if tc == qc {
                qi.next();
                score += 1;
                // Bonus for consecutive matches
                if prev_matched {
                    consecutive += 1;
                    score += consecutive * 2;
                } else {
                    consecutive = 0;
                }
                // Bonus for matching at start or after separator
                if i == 0
                    || target
                        .as_bytes()
                        .get(i.wrapping_sub(1))
                        .map(|&b| b == b'-' || b == b'_' || b == b' ' || b == b'/')
                        .unwrap_or(false)
                {
                    score += 5;
                }
                prev_matched = true;
            } else {
                prev_matched = false;
                consecutive = 0;
            }
        }
    }

    // All query chars must match
    if qi.peek().is_some() {
        return 0;
    }

    score
}

/// Filter and sort items by fuzzy match score.
pub fn fuzzy_filter<'a>(query: &str, items: &'a [String]) -> Vec<(usize, &'a str, u32)> {
    let mut scored: Vec<(usize, &str, u32)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let score = fuzzy_score(query, item);
            if score > 0 {
                Some((i, item.as_str(), score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.2.cmp(&a.2));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(fuzzy_score("rust", "rust") > fuzzy_score("rust", "rustic"));
    }

    #[test]
    fn test_substring_match() {
        assert!(fuzzy_score("api", "api-design") > 0);
    }

    #[test]
    fn test_subsequence_match() {
        assert!(fuzzy_score("md", "markdown") > 0);
    }

    #[test]
    fn test_no_match() {
        assert_eq!(fuzzy_score("xyz", "abc"), 0);
    }

    #[test]
    fn test_empty_query() {
        assert!(fuzzy_score("", "anything") > 0);
    }
}
