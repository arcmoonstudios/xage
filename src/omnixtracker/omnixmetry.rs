// src/omnixtracker/omnixmetry.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXTRACKER]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::constants::{PROMETHEUS_LISTENER, PROMETHEUS_TEST_LISTENER, INITIAL_LOG_LEVEL, LOG_FILE_PATH};
use tracing_subscriber::{Layer, Registry, EnvFilter};
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::{Event, Level, Metadata, Subscriber};
use anyhow::{Context, Result as AnyhowResult};
use tracing_subscriber::prelude::*;
use std::fmt::Write as FmtWrite;
use once_cell::sync::OnceCell;
use std::net::TcpListener;
use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{Local, Duration};
use regex::Regex;
use colored::*;
use plotters::prelude::*;
use std::collections::{VecDeque, HashMap};

static PROMETHEUS_RECORDER: OnceCell<()> = OnceCell::new();

#[derive(Clone)]
pub struct OmniXMetry {
    log_file: Arc<RwLock<Option<BufWriter<File>>>>,
    log_level: Arc<RwLock<Level>>,
    metrics_data: Arc<RwLock<MetricsData>>,
}

struct MetricsData {
    counters: HashMap<String, VecDeque<(chrono::DateTime<Local>, u64)>>,
    gauges: HashMap<String, VecDeque<(chrono::DateTime<Local>, f64)>>,
    histograms: HashMap<String, VecDeque<(chrono::DateTime<Local>, f64)>>,
}

impl MetricsData {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    fn add_counter(&mut self, key: String, value: u64) {
        let entry = self.counters.entry(key).or_insert_with(VecDeque::new);
        entry.push_back((Local::now(), value));
        if entry.len() > 100 {
            entry.pop_front();
        }
    }

    fn add_gauge(&mut self, key: String, value: f64) {
        let entry = self.gauges.entry(key).or_insert_with(VecDeque::new);
        entry.push_back((Local::now(), value));
        if entry.len() > 100 {
            entry.pop_front();
        }
    }

    fn add_histogram(&mut self, key: String, value: f64) {
        let entry = self.histograms.entry(key).or_insert_with(VecDeque::new);
        entry.push_back((Local::now(), value));
        if entry.len() > 100 {
            entry.pop_front();
        }
    }
}

impl OmniXMetry {
    pub fn init() -> AnyhowResult<Self> {
        PROMETHEUS_RECORDER.get_or_try_init(|| {
            let listener_result = if cfg!(test) {
                TcpListener::bind(&*PROMETHEUS_TEST_LISTENER)
                    .context("Failed to bind to test port")
            } else {
                TcpListener::bind(&*PROMETHEUS_LISTENER)
                    .context("Failed to bind to configured Prometheus listener")
            };

            let listener = listener_result?;
            println!("Prometheus listening on: {}", listener.local_addr()?);

            PrometheusBuilder::new()
                .with_http_listener(listener.local_addr()?)
                .install_recorder()
                .context("Failed to set global Prometheus recorder")?;

            Ok::<(), anyhow::Error>(())
        }).context("Failed to initialize Prometheus recorder")?;

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*LOG_FILE_PATH)
            .context("Failed to open log file")?;

        let buffered_file = BufWriter::new(log_file);

        Ok(Self {
            log_level: Arc::new(RwLock::new(*INITIAL_LOG_LEVEL)),
            log_file: Arc::new(RwLock::new(Some(buffered_file))),
            metrics_data: Arc::new(RwLock::new(MetricsData::new())),
        })
    }

    pub fn set_log_level(&self, level: Level) {
        let mut log_level = self.log_level.write();
        *log_level = level;
    }

    pub fn get_log_level(&self) -> Level {
        *self.log_level.read()
    }

    pub fn is_log_file_initialized(&self) -> bool {
        self.log_file.read().is_some()
    }

    pub fn increment_counter(&self, key_name: String, value: u64) {
        let counter = metrics::counter!(key_name.clone(), "value" => value.to_string());
        counter.increment(value);
        self.metrics_data.write().add_counter(key_name, value);
    }

    pub fn update_gauge(&self, key_name: String, value: f64) {
        let gauge = metrics::gauge!(key_name.clone(), "value" => value.to_string());
        gauge.set(value);
        self.metrics_data.write().add_gauge(key_name, value);
    }

    pub fn record_histogram(&self, key_name: String, value: f64) {
        let histogram = metrics::histogram!(key_name.clone(), "value" => value.to_string());
        histogram.record(value);
        self.metrics_data.write().add_histogram(key_name, value);
    }

    pub fn rotate_log_file(&self) -> AnyhowResult<()> {
        let mut log_file_lock = self.log_file.write();
        if let Some(mut file) = log_file_lock.take() {
            file.flush().context("Failed to flush log file before rotation")?;
            drop(file);

            let xdocs_path = Path::new("Xdocs");
            std::fs::create_dir_all(&xdocs_path)
                .context("Failed to create Xdocs directory")?;

            let old_log_path = Path::new(&*LOG_FILE_PATH);
            if old_log_path.exists() {
                let new_log_path = self.generate_new_log_path(xdocs_path)?;
                std::fs::rename(&old_log_path, &new_log_path)
                    .context("Failed to rename log file")?;
            } else {
                println!("Warning: Log file doesn't exist. Creating a new one.");
            }
        }

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*LOG_FILE_PATH)
            .context("Failed to open new log file")?;

        let buffered_file = BufWriter::new(new_file);
        *log_file_lock = Some(buffered_file);

        Ok(())
    }

    fn generate_new_log_path(&self, xdocs_path: &Path) -> AnyhowResult<PathBuf> {
        let date_time = Local::now().format("%m-%d-%Y_%H-%M");
        let base_name = format!("{}", date_time);

        let regex = Regex::new(&format!(r"^{}_(\d{{3}})\.log$", regex::escape(&base_name)))
            .context("Failed to create regex for log file naming")?;

        let mut highest_number = 0;
        for entry in std::fs::read_dir(xdocs_path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap_or_default();
            if let Some(captures) = regex.captures(&file_name) {
                if let Some(number_match) = captures.get(1) {
                    if let Ok(number) = number_match.as_str().parse::<u32>() {
                        highest_number = highest_number.max(number);
                    }
                }
            }
        }

        let new_number = highest_number + 1;
        let new_file_name = format!("{}_{:03}.log", base_name, new_number);
        Ok(xdocs_path.join(new_file_name))
    }

    pub fn write_log(&self, log_entry: &str) -> std::io::Result<()> {
        if let Some(ref mut file) = *self.log_file.write() {
            writeln!(file, "{}", log_entry)?;
            file.flush()?;
        }
        Ok(())
    }

    pub fn generate_metrics_chart(&self, metric_type: &str, metric_name: &str) -> AnyhowResult<()> {
        let metrics_data = self.metrics_data.read();
        let data = match metric_type {
            "counter" => metrics_data.counters.get(metric_name).map(|d| d.iter().map(|(t, &v)| (*t, v as f64)).collect::<Vec<_>>()),
            "gauge" => metrics_data.gauges.get(metric_name).map(|d| d.iter().map(|(t, &v)| (*t, v)).collect::<Vec<_>>()),
            "histogram" => metrics_data.histograms.get(metric_name).map(|d| d.iter().map(|(t, &v)| (*t, v)).collect::<Vec<_>>()),
            _ => return Err(anyhow::anyhow!("Invalid metric type")),
        };

        let data = data.ok_or_else(|| anyhow::anyhow!("No data for the specified metric"))?;

        let root = BitMapBackend::new(&format!("{}_{}.png", metric_type, metric_name), (800, 600))
            .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(format!("{} over time", metric_name), ("sans-serif", 40).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                data.first().unwrap().0..data.last().unwrap().0,
                0f64..data.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max),
            )?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                data.iter().map(|(t, v)| (*t, *v)),
                &RED,
            ))?
            .label(metric_name)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        Ok(())
    }
}

impl<S> Layer<S> for OmniXMetry
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if *event.metadata().level() <= self.get_log_level() {
            let level_str = match *event.metadata().level() {
                Level::ERROR => "ERROR".red(),
                Level::WARN => "WARN ".yellow(),
                Level::INFO => "INFO ".green(),
                Level::DEBUG => "DEBUG".blue(),
                Level::TRACE => "TRACE".magenta(),
            };

            let mut fields = String::new();
            {
                let mut visitor = FieldVisitor { output: &mut fields };
                event.record(&mut visitor);
            }

            let log_entry = format!(
                "{} [{}] {}: {}",
                Local::now().format("%B, %d %Y @ %I:%M %p"),
                level_str,
                event.metadata().target(),
                fields
            );

            println!("{}", log_entry);

            if let Err(e) = self.write_log(&log_entry) {
                eprintln!("Failed to write to log file: {}", e);
            }
        }
    }

    fn enabled(
        &self,
        metadata: &Metadata<'_>,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        *metadata.level() <= self.get_log_level()
    }
}

struct FieldVisitor<'a> {
    output: &'a mut String,
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if !self.output.is_empty() {
            self.output.push_str(", ");
        }
        let _ = write!(self.output, "{} = {:?}", field.name(), value);
    }
}

pub fn setup_global_subscriber(omnixmetry: OmniXMetry) -> AnyhowResult<()> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    let subscriber = Registry::default().with(env_filter).with(omnixmetry);
    tracing::subscriber::set_global_default(subscriber).context("Failed to set global subscriber")?;
    Ok(())
}