use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use std::io;

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: Level,
    pub format: LogFormat,
    pub output: LogOutput,
    pub include_file_location: bool,
    pub include_thread_names: bool,
    pub include_span_events: bool,
    pub filter: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: LogFormat::Pretty,
            output: LogOutput::Stdout,
            include_file_location: true,
            include_thread_names: false,
            include_span_events: false,
            filter: None,
        }
    }
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// 人类可读的格式
    Pretty,
    /// 紧凑格式
    Compact,
    /// JSON 格式
    Json,
}

/// 日志输出目标
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
    /// 文件
    File(String),
    /// 同时输出到多个目标
    Multiple(Vec<LogOutput>),
}

/// 设置日志系统
pub fn setup_logging(config: LoggingConfig) -> anyhow::Result<()> {
    let env_filter = if let Some(filter) = &config.filter {
        EnvFilter::try_new(filter)?
    } else {
        EnvFilter::from_default_env()
            .add_directive(format!("ai_commit={}", config.level).parse()?)
    };

    match config.output {
        LogOutput::Stdout => {
            setup_stdout_logging(config, env_filter)?;
        },
        LogOutput::Stderr => {
            setup_stderr_logging(config, env_filter)?;
        },
        LogOutput::File(path) => {
            setup_file_logging(config, env_filter, path)?;
        },
        LogOutput::Multiple(outputs) => {
            setup_multiple_logging(config, env_filter, outputs)?;
        },
    }

    Ok(())
}

fn setup_stdout_logging(config: LoggingConfig, env_filter: EnvFilter) -> anyhow::Result<()> {
    let fmt_layer = create_fmt_layer(&config, io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn setup_stderr_logging(config: LoggingConfig, env_filter: EnvFilter) -> anyhow::Result<()> {
    let fmt_layer = create_fmt_layer(&config, io::stderr);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn setup_file_logging(config: LoggingConfig, env_filter: EnvFilter, path: String) -> anyhow::Result<()> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    let fmt_layer = create_fmt_layer(&config, move || file.try_clone().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

fn setup_multiple_logging(config: LoggingConfig, env_filter: EnvFilter, outputs: Vec<LogOutput>) -> anyhow::Result<()> {
    // 对于多输出，我们创建多个层
    let mut layers = Vec::new();

    for output in outputs {
        match output {
            LogOutput::Stdout => {
                let layer = create_fmt_layer(&config, io::stdout);
                layers.push(layer.boxed());
            },
            LogOutput::Stderr => {
                let layer = create_fmt_layer(&config, io::stderr);
                layers.push(layer.boxed());
            },
            LogOutput::File(path) => {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)?;
                let layer = create_fmt_layer(&config, move || file.try_clone().unwrap());
                layers.push(layer.boxed());
            },
            LogOutput::Multiple(_) => {
                // 避免递归
                continue;
            },
        }
    }

    // 由于 tracing_subscriber 的限制，我们只能使用第一个层
    // 在实际实现中，可能需要使用更复杂的方法来支持多输出
    if let Some(first_layer) = layers.into_iter().next() {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(first_layer)
            .init();
    }

    Ok(())
}

fn create_fmt_layer<W>(config: &LoggingConfig, make_writer: W) -> impl Layer<tracing_subscriber::Registry>
where
    W: for<'writer> fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let mut layer = fmt::layer()
        .with_writer(make_writer)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(config.include_thread_names)
        .with_thread_names(config.include_thread_names);

    if config.include_file_location {
        layer = layer.with_file(true).with_line_number(true);
    }

    if config.include_span_events {
        layer = layer.with_span_events(FmtSpan::FULL);
    }

    match config.format {
        LogFormat::Pretty => layer.pretty().boxed(),
        LogFormat::Compact => layer.compact().boxed(),
        LogFormat::Json => layer.json().boxed(),
    }
}

/// 创建结构化日志记录器
pub struct StructuredLogger {
    correlation_id: Option<String>,
    component: String,
}

impl StructuredLogger {
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            correlation_id: None,
            component: component.into(),
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn info(&self, message: &str, fields: Option<&[(&str, &str)]>) {
        let span = tracing::info_span!(
            "structured_log",
            component = %self.component,
            correlation_id = self.correlation_id.as_deref().unwrap_or(""),
        );
        let _enter = span.enter();

        if let Some(fields) = fields {
            let mut event = tracing::info!("{}", message);
            for (key, value) in fields {
                event = tracing::info!(%key = %value, "{}", message);
            }
        } else {
            tracing::info!("{}", message);
        }
    }

    pub fn warn(&self, message: &str, fields: Option<&[(&str, &str)]>) {
        let span = tracing::warn_span!(
            "structured_log",
            component = %self.component,
            correlation_id = self.correlation_id.as_deref().unwrap_or(""),
        );
        let _enter = span.enter();

        if let Some(fields) = fields {
            for (key, value) in fields {
                tracing::warn!(%key = %value, "{}", message);
            }
        } else {
            tracing::warn!("{}", message);
        }
    }

    pub fn error(&self, message: &str, error: Option<&dyn std::error::Error>, fields: Option<&[(&str, &str)]>) {
        let span = tracing::error_span!(
            "structured_log",
            component = %self.component,
            correlation_id = self.correlation_id.as_deref().unwrap_or(""),
        );
        let _enter = span.enter();

        if let Some(err) = error {
            if let Some(fields) = fields {
                for (key, value) in fields {
                    tracing::error!(%key = %value, error = %err, "{}", message);
                }
            } else {
                tracing::error!(error = %err, "{}", message);
            }
        } else if let Some(fields) = fields {
            for (key, value) in fields {
                tracing::error!(%key = %value, "{}", message);
            }
        } else {
            tracing::error!("{}", message);
        }
    }

    pub fn debug(&self, message: &str, fields: Option<&[(&str, &str)]>) {
        let span = tracing::debug_span!(
            "structured_log",
            component = %self.component,
            correlation_id = self.correlation_id.as_deref().unwrap_or(""),
        );
        let _enter = span.enter();

        if let Some(fields) = fields {
            for (key, value) in fields {
                tracing::debug!(%key = %value, "{}", message);
            }
        } else {
            tracing::debug!("{}", message);
        }
    }
}

/// 性能监控宏
#[macro_export]
macro_rules! measure_time {
    ($operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!(
            operation = $operation,
            duration_ms = duration.as_millis(),
            "操作完成"
        );
        result
    }};
}

/// 审查操作跟踪
pub struct ReviewTracker {
    operation_id: String,
    start_time: std::time::Instant,
    logger: StructuredLogger,
}

impl ReviewTracker {
    pub fn new(operation: impl Into<String>, correlation_id: Option<String>) -> Self {
        let operation_id = uuid::Uuid::new_v4().to_string();
        let logger = StructuredLogger::new("review_tracker")
            .with_correlation_id(correlation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()));

        logger.info(
            "开始审查操作",
            Some(&[
                ("operation", &operation.into()),
                ("operation_id", &operation_id),
            ])
        );

        Self {
            operation_id,
            start_time: std::time::Instant::now(),
            logger,
        }
    }

    pub fn log_progress(&self, stage: &str, progress: f32) {
        self.logger.info(
            "审查进度更新",
            Some(&[
                ("operation_id", &self.operation_id),
                ("stage", stage),
                ("progress", &format!("{:.1}%", progress * 100.0)),
            ])
        );
    }

    pub fn log_error(&self, stage: &str, error: &dyn std::error::Error) {
        self.logger.error(
            "审查操作出错",
            Some(error),
            Some(&[
                ("operation_id", &self.operation_id),
                ("stage", stage),
            ])
        );
    }

    pub fn complete(self, success: bool, files_processed: usize) {
        let duration = self.start_time.elapsed();

        if success {
            self.logger.info(
                "审查操作完成",
                Some(&[
                    ("operation_id", &self.operation_id),
                    ("duration_ms", &duration.as_millis().to_string()),
                    ("files_processed", &files_processed.to_string()),
                    ("success", "true"),
                ])
            );
        } else {
            self.logger.error(
                "审查操作失败",
                None,
                Some(&[
                    ("operation_id", &self.operation_id),
                    ("duration_ms", &duration.as_millis().to_string()),
                    ("files_processed", &files_processed.to_string()),
                    ("success", "false"),
                ])
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(matches!(config.output, LogOutput::Stdout));
        assert!(config.include_file_location);
        assert!(!config.include_thread_names);
    }

    #[test]
    fn test_structured_logger_creation() {
        let logger = StructuredLogger::new("test_component");
        assert_eq!(logger.component, "test_component");
        assert!(logger.correlation_id.is_none());

        let logger_with_id = logger.with_correlation_id("test-id");
        assert_eq!(logger_with_id.correlation_id, Some("test-id".to_string()));
    }

    #[test]
    fn test_review_tracker_creation() {
        let tracker = ReviewTracker::new("test_operation", Some("test-correlation".to_string()));
        assert!(!tracker.operation_id.is_empty());
        assert_eq!(tracker.logger.component, "review_tracker");
    }
}