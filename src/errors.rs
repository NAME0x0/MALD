use crossterm::style::Stylize;

/// A user-facing error with a suggestion and optional help topic.
pub struct ContextualError {
    pub message: String,
    pub suggestion: Option<String>,
    pub help_topic: Option<String>,
}

impl ContextualError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            suggestion: None,
            help_topic: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_help_topic(mut self, topic: impl Into<String>) -> Self {
        self.help_topic = Some(topic.into());
        self
    }

    /// Print the error with colors to stderr.
    pub fn print(&self) {
        eprintln!("{} {}", "error:".red().bold(), self.message.as_str().red());
        if let Some(ref suggestion) = self.suggestion {
            eprintln!("  {} {}", "hint:".cyan().bold(), suggestion.as_str().cyan());
        }
        if let Some(ref topic) = self.help_topic {
            eprintln!(
                "  {}",
                format!("Run `mald help-topic {topic}` for more info").dark_grey()
            );
        }
        eprintln!("  {}", "Run `mald doctor` to check your setup.".dark_grey());
    }
}

/// Try to extract a ContextualError from an anyhow::Error chain.
/// Returns None if the error doesn't contain contextual info.
pub fn extract_contextual(err: &anyhow::Error) -> Option<&ContextualError> {
    err.downcast_ref::<ContextualErrorWrapper>().map(|w| &w.0)
}

/// Wrapper to make ContextualError usable with anyhow.
#[derive(Debug)]
pub struct ContextualErrorWrapper(pub ContextualError);

impl std::fmt::Debug for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for ContextualErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message)
    }
}

impl std::error::Error for ContextualErrorWrapper {}

/// Create an anyhow::Error from a ContextualError.
pub fn contextual(err: ContextualError) -> anyhow::Error {
    anyhow::Error::new(ContextualErrorWrapper(err))
}

/// Convenience macro-like helper: create a contextual bail.
pub fn bail_ctx(message: impl Into<String>, suggestion: impl Into<String>) -> anyhow::Error {
    contextual(ContextualError::new(message).with_suggestion(suggestion))
}

pub fn bail_ctx_topic(
    message: impl Into<String>,
    suggestion: impl Into<String>,
    topic: impl Into<String>,
) -> anyhow::Error {
    contextual(
        ContextualError::new(message)
            .with_suggestion(suggestion)
            .with_help_topic(topic),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextual_error_creation() {
        let err = ContextualError::new("test error")
            .with_suggestion("try this")
            .with_help_topic("ai");
        assert_eq!(err.message, "test error");
        assert_eq!(err.suggestion.as_deref(), Some("try this"));
        assert_eq!(err.help_topic.as_deref(), Some("ai"));
    }

    #[test]
    fn test_bail_ctx() {
        let err = bail_ctx("something failed", "do this instead");
        let wrapper = err.downcast_ref::<ContextualErrorWrapper>().unwrap();
        assert_eq!(wrapper.0.message, "something failed");
        assert_eq!(wrapper.0.suggestion.as_deref(), Some("do this instead"));
    }

    #[test]
    fn test_contextual_error_print_no_panic() {
        // Just ensure print doesn't panic (output goes to stderr)
        let err = ContextualError::new("test")
            .with_suggestion("hint")
            .with_help_topic("ai");
        err.print();
    }

    #[test]
    fn test_extract_contextual() {
        let err = bail_ctx("msg", "hint");
        let ctx = extract_contextual(&err).unwrap();
        assert_eq!(ctx.message, "msg");
    }

    #[test]
    fn test_extract_contextual_none() {
        let err = anyhow::anyhow!("plain error");
        assert!(extract_contextual(&err).is_none());
    }
}
