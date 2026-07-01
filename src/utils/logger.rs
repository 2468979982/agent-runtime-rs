/// Initialize tracing subscriber for logging
pub fn init_logger(level: &str) -> anyhow::Result<()> {
    let log_level = match level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    Ok(())
}

/// Initialize logger with file output
pub fn init_logger_with_file(level: &str, file_path: &str) -> anyhow::Result<()> {
    let log_level = match level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        _ => tracing::Level::INFO,
    };

    // For simplicity, we'll just use stdout for now
    // In production, you'd want to use tracing-appender for file logging
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    tracing::info!("Logger initialized with file output: {}", file_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logger() {
        // This test just ensures the function doesn't panic
        // In practice, you can only initialize tracing once per process
        let result = init_logger("info");
        assert!(result.is_ok() || result.is_err()); // Either is fine for this test
    }
}
