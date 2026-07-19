use crate::logger::*;
use async_trait::async_trait;
use chrono::Local;
use config::ScreenConfig;
use data_modalities::screen::ScreenFrame;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use image::GenericImageView;
use image::ImageReader;
use lifelog_core::Utc;
use lifelog_core::Uuid;
use lifelog_types::to_pb_ts;
use prost::Message;
use std::io::Cursor;

use std::sync::Arc;

use crate::data_source::{BufferedSource, DataSource, DataSourceHandle, DiskBufferedSource};
use lifelog_core::LifelogError;
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);
/// Ensures the WebP-vs-PNG size comparison is logged exactly once per process.
static SIZE_LOGGED: AtomicBool = AtomicBool::new(false);

/// Encode a decoded image to lossy WebP at the given quality (0-100). grim has
/// no WebP output, so screen frames are captured as PNG and transcoded here
/// in-process.
fn encode_webp(img: &image::DynamicImage, quality: f32) -> Result<Vec<u8>, LifelogError> {
    let encoder = webp::Encoder::from_image(img).map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("WebP encoder init failed: {e}"),
        ))
    })?;
    Ok(encoder.encode(quality).to_vec())
}

#[derive(Debug, Clone)]
pub struct ScreenDataSource {
    config: ScreenConfig,
    logger: ScreenLogger,
    pub buffer: Arc<DiskBuffer>,
}

impl ScreenDataSource {
    pub fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        let logger = ScreenLogger::new(config.clone());
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(ScreenDataSource {
            config,
            logger: logger?,
            buffer: Arc::new(buffer),
        })
    }

    pub async fn get_data(&mut self) -> Result<Vec<ScreenFrame>, LifelogError> {
        let (_, raw_items) = self.buffer.peek_chunk(100).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let mut frames = Vec::new();
        for raw in raw_items {
            if let Ok(frame) = ScreenFrame::decode(raw.as_slice()) {
                frames.push(frame);
            }
        }
        Ok(frames)
    }

    async fn process_and_store(
        &self,
        image_data_bytes: Vec<u8>,
        source_output: String,
    ) -> Result<(), LifelogError> {
        let ts = Utc::now();

        let img = ImageReader::new(Cursor::new(&image_data_bytes))
            .with_guessed_format()
            .map_err(|e| LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .decode()
            .map_err(|e| LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let (width, height) = img.dimensions();

        // grim can't emit WebP, so transcode the captured PNG to lossy WebP
        // in-process. Config quality 0 means "unset" -> default 80.
        let quality = match self.config.webp_quality {
            0 => 80.0,
            q => q.min(100) as f32,
        };
        let webp_bytes = encode_webp(&img, quality)?;

        // One-shot WebP-vs-PNG size comparison for the record: the PNG grim
        // produced is what we'd otherwise store, so this is the real delta.
        if !SIZE_LOGGED.swap(true, Ordering::SeqCst) {
            let png_len = image_data_bytes.len();
            let webp_len = webp_bytes.len();
            tracing::info!(
                png_bytes = png_len,
                webp_bytes = webp_len,
                quality,
                ratio = format!("{:.1}%", 100.0 * webp_len as f64 / png_len.max(1) as f64),
                "screen capture: WebP vs PNG size"
            );
        }

        let timestamp = to_pb_ts(ts);
        let captured = ScreenFrame {
            uuid: Uuid::new_v4().to_string(),
            width,
            height,
            image_bytes: webp_bytes,
            timestamp,
            mime_type: "image/webp".to_string(),
            source_output,
            t_device: timestamp,
            t_canonical: timestamp,
            t_end: timestamp,
            ..Default::default()
        };

        let mut buf = Vec::new();
        captured.encode(&mut buf).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Prost encode error: {}", e),
            ))
        })?;

        self.buffer.append(&buf).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        tracing::debug!("ScreenDataSource: Stored screen capture in WAL");
        Ok(())
    }

    pub async fn clear_buffer(&self) -> Result<(), LifelogError> {
        // Mark all current data as committed
        let start = self.buffer.get_committed_offset().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        let size = self.buffer.get_uncommitted_size().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        self.buffer.commit_offset(start + size).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }
}

#[async_trait]
impl DataSource for ScreenDataSource {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        ScreenDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(DiskBufferedSource::new(
            "screen",
            self.buffer.clone(),
        )))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("ScreenDataSource: Start called but task is already running.");
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("ScreenDataSource: Starting data source task to store in WAL");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "ScreenDataSource (WAL) background task finished");
            task_result
        });

        tracing::info!("ScreenDataSource: Data source task (WAL) started successfully");
        Ok(DataSourceHandle { join: join_handle })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        // FIXME, actually implmenet stop handles
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        while RUNNING.load(Ordering::SeqCst) {
            match self.logger.capture_frames().await {
                Ok(frames) => {
                    // One ScreenFrame per powered-on output, each its own uuid,
                    // all sharing this tick's capture time.
                    for (image_data_bytes, source_output) in frames {
                        if let Err(e) = self
                            .process_and_store(image_data_bytes, source_output)
                            .await
                        {
                            tracing::error!(error = %e, "ScreenDataSource: Failed to process/store frame, continuing");
                        }
                    }
                }
                Err(e) => {
                    let display_off = matches!(&e, LifelogError::Io(io) if io.kind() == std::io::ErrorKind::WouldBlock);
                    if display_off {
                        tracing::debug!("ScreenDataSource: display off; skipping capture");
                    } else {
                        tracing::error!(error = %e, "ScreenDataSource: Failed to capture screen data for WAL store");
                    }
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        tracing::info!("ScreenDataSource: WAL run loop finished");
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}
#[derive(Clone, Debug)]
pub struct ScreenLogger {
    config: ScreenConfig,
}

impl ScreenLogger {
    pub fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        Ok(ScreenLogger { config })
    }

    pub fn setup(&self) -> Result<LoggerHandle, LifelogError> {
        DataLogger::setup(self, self.config.clone())
    }

    async fn capture_screenshot_data(&self) -> Result<Vec<u8>, LifelogError> {
        // let temp_file = NamedTempFile::new_in(env::temp_dir())?.into_temp_path();
        // let temp_file_path_str = temp_file.to_str().ok_or_else(|| LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid temp file path")))?;

        let now = Local::now();
        let _ts = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1e9;
        let ts_fmt = now.format(&self.config.timestamp_format);
        let out = format!("{}/{}.png", self.config.output_dir, ts_fmt);
        tracing::debug!(path = %out, "Capturing screenshot");

        #[cfg(target_os = "macos")]
        {
            tokio::time::timeout(
                Duration::from_secs(10),
                Command::new("screencapture")
                    .arg("-x")
                    .arg("-t")
                    .arg("png")
                    .arg(&out)
                    .status(),
            )
            .await
            .map_err(|_| {
                LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Screenshot timed out",
                ))
            })?
            .map_err(LifelogError::Io)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut cmd = Command::new(&self.config.program);
            match pick_capture_target().await {
                CaptureTarget::Nothing => {
                    return Err(LifelogError::Io(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "all outputs DPMS-off; nothing to capture",
                    )));
                }
                // grim blocks until timeout if ANY output is DPMS-off, so
                // when only some are on, capture the focused/on one instead
                // of the whole space. (-o is grim-specific.)
                CaptureTarget::Output(name) if self.config.program.ends_with("grim") => {
                    cmd.arg("-o").arg(name);
                }
                CaptureTarget::Output(_) | CaptureTarget::WholeSpace => {}
            }
            let status = tokio::time::timeout(
                Duration::from_secs(10),
                cmd.arg(&out).status(),
            )
            .await
            .map_err(|_| {
                tracing::error!(program = %self.config.program, "Screenshot program timed out");
                LifelogError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "Screenshot timed out"))
            })?
            .map_err(|e| {
                tracing::error!(program = %self.config.program, error = %e, "Failed to start screenshot program");
                LifelogError::Io(e)
            })?;

            if !status.success() {
                tracing::error!(program = %self.config.program, status = ?status, path = %out, "Screenshot program exited with error");
                return Err(LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Screenshot program failed with status {:?}", status),
                )));
            }
        }

        let image_data = tokio::fs::read(&out).await.map_err(|e| {
            tracing::error!(path = %out, error = %e, "Failed to read captured screenshot file");
            LifelogError::Io(e)
        })?;

        if let Err(e) = tokio::fs::remove_file(&out).await {
            tracing::warn!(error = %e, "Failed to delete temporary screenshot");
        }

        Ok(image_data)
    }

    /// Capture one image per powered-on output. On Hyprland with grim, each
    /// dpms-on output is captured separately (`grim -o <name>`) and tagged with
    /// its name. Non-grim programs, macOS, or non-Hyprland sessions fall back to
    /// a single whole-space capture tagged with an empty output. Every output
    /// off -> WouldBlock so the caller skips the tick.
    async fn capture_frames(&self) -> Result<Vec<(Vec<u8>, String)>, LifelogError> {
        #[cfg(not(target_os = "macos"))]
        if self.config.program.ends_with("grim") {
            match enumerate_capture_targets().await {
                CaptureTargets::Nothing => {
                    return Err(LifelogError::Io(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "all outputs DPMS-off; nothing to capture",
                    )));
                }
                CaptureTargets::Outputs(names) => {
                    let mut frames = Vec::with_capacity(names.len());
                    for name in &names {
                        match self.capture_to_bytes(Some(name)).await {
                            Ok(bytes) => frames.push((bytes, name.clone())),
                            Err(e) => {
                                tracing::error!(output = %name, error = %e, "Per-output capture failed; skipping this output")
                            }
                        }
                    }
                    if frames.is_empty() {
                        return Err(LifelogError::Io(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "every per-output capture failed",
                        )));
                    }
                    return Ok(frames);
                }
                // Non-Hyprland (hyprctl missing/failing): whole-space fallback below.
                CaptureTargets::WholeSpace => {}
            }
        }

        // Fallback: single whole-space capture, untagged output.
        let bytes = self.capture_to_bytes(None).await?;
        Ok(vec![(bytes, String::new())])
    }

    /// Capture a single frame to a temp file and return its raw bytes. `output`
    /// selects a specific grim output (`-o <name>`); None captures the whole
    /// space. The output name is folded into the temp filename so per-output
    /// captures in the same tick don't collide.
    async fn capture_to_bytes(&self, output: Option<&str>) -> Result<Vec<u8>, LifelogError> {
        let now = Local::now();
        let ts_fmt = now.format(&self.config.timestamp_format);
        let suffix = output.map(|o| format!("-{o}")).unwrap_or_default();
        let out = format!("{}/{}{}.png", self.config.output_dir, ts_fmt, suffix);
        tracing::debug!(path = %out, output = ?output, "Capturing screenshot");

        #[cfg(target_os = "macos")]
        {
            tokio::time::timeout(
                Duration::from_secs(10),
                Command::new("screencapture")
                    .arg("-x")
                    .arg("-t")
                    .arg("png")
                    .arg(&out)
                    .status(),
            )
            .await
            .map_err(|_| {
                LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Screenshot timed out",
                ))
            })?
            .map_err(LifelogError::Io)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut cmd = Command::new(&self.config.program);
            if let Some(name) = output {
                if self.config.program.ends_with("grim") {
                    cmd.arg("-o").arg(name);
                }
            }
            let status = tokio::time::timeout(Duration::from_secs(10), cmd.arg(&out).status())
                .await
                .map_err(|_| {
                    tracing::error!(program = %self.config.program, "Screenshot program timed out");
                    LifelogError::Io(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Screenshot timed out",
                    ))
                })?
                .map_err(|e| {
                    tracing::error!(program = %self.config.program, error = %e, "Failed to start screenshot program");
                    LifelogError::Io(e)
                })?;

            if !status.success() {
                tracing::error!(program = %self.config.program, status = ?status, path = %out, "Screenshot program exited with error");
                return Err(LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Screenshot program failed with status {:?}", status),
                )));
            }
        }

        let image_data = tokio::fs::read(&out).await.map_err(|e| {
            tracing::error!(path = %out, error = %e, "Failed to read captured screenshot file");
            LifelogError::Io(e)
        })?;

        if let Err(e) = tokio::fs::remove_file(&out).await {
            tracing::warn!(error = %e, "Failed to delete temporary screenshot");
        }

        Ok(image_data)
    }
}

/// Powered-on outputs to capture this tick.
enum CaptureTargets {
    /// Non-Hyprland session (hyprctl missing/failing): one whole-space capture.
    WholeSpace,
    /// Capture each of these powered-on outputs separately (one frame each).
    Outputs(Vec<String>),
    /// Every output is DPMS-off: skip this tick.
    Nothing,
}

/// Ask Hyprland for the set of powered-on outputs so each can be captured
/// separately. Non-Hyprland sessions (hyprctl missing/failing/empty) fall back
/// to a single whole-space capture.
async fn enumerate_capture_targets() -> CaptureTargets {
    let Ok(out) = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .await
    else {
        return CaptureTargets::WholeSpace;
    };
    if !out.status.success() {
        return CaptureTargets::WholeSpace;
    }
    let Ok(serde_json::Value::Array(mons)) = serde_json::from_slice(&out.stdout) else {
        return CaptureTargets::WholeSpace;
    };
    if mons.is_empty() {
        return CaptureTargets::WholeSpace;
    }
    let dpms_on =
        |m: &&serde_json::Value| m.get("dpmsStatus").and_then(|v| v.as_bool()) != Some(false);
    let names: Vec<String> = mons
        .iter()
        .filter(dpms_on)
        .filter_map(|m| {
            m.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    if names.is_empty() {
        CaptureTargets::Nothing
    } else {
        CaptureTargets::Outputs(names)
    }
}

enum CaptureTarget {
    /// All outputs on (or not a Hyprland session): capture the whole space.
    WholeSpace,
    /// Some outputs are DPMS-off: capture this powered-on output only.
    Output(String),
    /// Every output is DPMS-off: nothing to capture.
    Nothing,
}

/// grim blocks until timeout when any output is DPMS-off, so ask Hyprland
/// which outputs are powered before capturing. Non-Hyprland sessions
/// (hyprctl missing/failing) fall back to whole-space capture.
async fn pick_capture_target() -> CaptureTarget {
    let Ok(out) = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .await
    else {
        return CaptureTarget::WholeSpace;
    };
    if !out.status.success() {
        return CaptureTarget::WholeSpace;
    }
    let Ok(serde_json::Value::Array(mons)) = serde_json::from_slice(&out.stdout) else {
        return CaptureTarget::WholeSpace;
    };
    if mons.is_empty() {
        return CaptureTarget::WholeSpace;
    }
    let dpms_on =
        |m: &&serde_json::Value| m.get("dpmsStatus").and_then(|v| v.as_bool()) != Some(false);
    if mons.iter().all(|m| dpms_on(&m)) {
        return CaptureTarget::WholeSpace;
    }
    let focused_on = mons
        .iter()
        .filter(dpms_on)
        .find(|m| m.get("focused").and_then(|v| v.as_bool()) == Some(true));
    let any_on = mons.iter().find(dpms_on);
    match focused_on
        .or(any_on)
        .and_then(|m| m.get("name").and_then(|v| v.as_str()))
    {
        Some(name) => CaptureTarget::Output(name.to_string()),
        None => CaptureTarget::Nothing,
    }
}

#[async_trait]
impl DataLogger for ScreenLogger {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        ScreenLogger::new(config)
    }

    fn setup(&self, config: ScreenConfig) -> Result<LoggerHandle, LifelogError> {
        let logger = Self::new(config)?;
        let join = tokio::spawn(async move {
            let task_result = logger.run().await;

            tracing::info!(result = ?task_result, "Background task finished");

            task_result
        });

        Ok(LoggerHandle { join })
    }

    async fn run(&self) -> Result<(), LifelogError> {
        RUNNING.store(true, Ordering::SeqCst);
        while RUNNING.load(Ordering::SeqCst) {
            self.log_data().await?;
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        Ok(())
    }

    fn stop(&self) {
        RUNNING.store(false, Ordering::SeqCst);
    }

    async fn log_data(&self) -> Result<Vec<u8>, LifelogError> {
        self.capture_screenshot_data().await
    }
}

#[cfg(test)]
mod tests {
    use super::encode_webp;
    use image::{DynamicImage, ImageFormat, RgbImage};
    use std::io::Cursor;

    /// A continuous-tone ("plasma") image standing in for a real screenshot:
    /// smooth enough that lossy WebP crushes it, complex enough that PNG's
    /// predictive filters can't. A flat gradient would let PNG compress to
    /// near-nothing and understate WebP's win, so we deliberately avoid one.
    fn synthetic_screenshot(w: u32, h: u32) -> DynamicImage {
        let mut img = RgbImage::new(w, h);
        for (x, y, px) in img.enumerate_pixels_mut() {
            let (xf, yf) = (x as f64, y as f64);
            let f = |a: f64, b: f64, phase: f64| {
                (128.0 + 90.0 * (xf * a + yf * b + phase).sin()).clamp(0.0, 255.0) as u8
            };
            *px = image::Rgb([
                f(0.06, 0.021, 0.0),
                f(0.017, 0.053, 1.7),
                f(0.041, 0.037, 3.1),
            ]);
        }
        DynamicImage::ImageRgb8(img)
    }

    /// Acceptance: lossy WebP must be well under 30% of the PNG grim would emit.
    #[test]
    fn webp_is_under_30pct_of_png() {
        let img = synthetic_screenshot(960, 600);

        let mut png = Vec::new();
        img.write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .expect("PNG encode");

        let webp = encode_webp(&img, 80.0).expect("WebP encode");

        let ratio = 100.0 * webp.len() as f64 / png.len() as f64;
        println!(
            "png={} bytes, webp(q80)={} bytes, ratio={:.1}%",
            png.len(),
            webp.len(),
            ratio
        );
        assert!(
            webp.len() * 100 < png.len() * 30,
            "WebP should be <30% of PNG: png={} webp={} ({:.1}%)",
            png.len(),
            webp.len(),
            ratio
        );
    }

    /// The lossy WebP we store must decode back to the original dimensions via
    /// the same `image` crate path OCR uses server-side.
    #[test]
    fn webp_decodes_to_original_dims() {
        let img = synthetic_screenshot(320, 240);
        let webp = encode_webp(&img, 80.0).expect("WebP encode");

        let decoded = image::ImageReader::new(Cursor::new(&webp))
            .with_guessed_format()
            .expect("guess format")
            .decode()
            .expect("decode WebP");

        assert_eq!((decoded.width(), decoded.height()), (320, 240));
    }
}
