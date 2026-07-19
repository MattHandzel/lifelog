use crate::data_source::{BufferedSource, DataSource, DataSourceHandle, DiskBufferedSource};
use async_trait::async_trait;
use lifelog_core::LifelogError;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

/// The real, per-modality work of a buffered polling source. A module implements
/// only this: the generic [`PollingSource`] supplies the WAL buffer, the
/// per-instance running flag, the lifecycle, and the run loop. Adding a new
/// polling modality is therefore one `Capture` impl plus one registry line.
#[async_trait]
pub trait Capture: Send + Sync + 'static {
    /// The modality's config type (a prost message from `CollectorConfig`).
    type Config: Clone + Debug + Send + Sync + 'static;

    /// Build the capturer from its config. Fails only on unrecoverable misconfig.
    fn from_config(config: Self::Config) -> Result<Self, LifelogError>
    where
        Self: Sized;

    /// Directory whose `buffer/` subdir holds this source's WAL.
    fn output_dir(&self) -> &str;

    /// Stable stream id — the modality name on the wire.
    fn stream_id(&self) -> &str;

    /// Poll interval between `capture()` calls.
    fn interval(&self) -> Duration;

    /// One-time setup run once before the loop. Returning `Err` aborts the run
    /// (the task exits), matching sources that validate prerequisites at startup.
    async fn init(&self) -> Result<(), LifelogError> {
        Ok(())
    }

    /// Produce zero or more encoded (prost-serialized) frames for this tick.
    /// An empty vec means "nothing to store this tick".
    async fn capture(&self) -> Result<Vec<Vec<u8>>, LifelogError>;
}

/// Generic WAL-backed polling source. Owns the buffer, a per-instance running
/// flag (so stopping one instance never stops a sibling — the bug the old
/// module-global `static RUNNING` had), and the poll loop.
pub struct PollingSource<C: Capture> {
    capture: Arc<C>,
    buffer: Arc<DiskBuffer>,
    running: Arc<AtomicBool>,
    config: C::Config,
}

impl<C: Capture> Clone for PollingSource<C> {
    fn clone(&self) -> Self {
        Self {
            capture: self.capture.clone(),
            buffer: self.buffer.clone(),
            running: self.running.clone(),
            config: self.config.clone(),
        }
    }
}

#[async_trait]
impl<C: Capture> DataSource for PollingSource<C> {
    type Config = C::Config;

    fn new(config: C::Config) -> Result<Self, LifelogError> {
        let capture = C::from_config(config.clone())?;
        let buffer_path = std::path::Path::new(capture.output_dir()).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(Self {
            capture: Arc::new(capture),
            buffer: Arc::new(buffer),
            running: Arc::new(AtomicBool::new(false)),
            config,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(DiskBufferedSource::new(
            self.capture.stream_id(),
            self.buffer.clone(),
        )))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        let stream = self.capture.stream_id().to_string();
        tracing::info!(source = %stream, "PollingSource: starting data source task");
        let this = self.clone();

        let join_handle = tokio::spawn(async move {
            let task_result = this.run().await;
            tracing::info!(source = %stream, result = ?task_result, "PollingSource background task finished");
            task_result
        });

        Ok(DataSourceHandle { join: join_handle })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        self.capture.init().await?;

        while self.running.load(Ordering::SeqCst) {
            match self.capture.capture().await {
                Ok(frames) => {
                    for buf in frames {
                        if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!(
                                source = %self.capture.stream_id(),
                                "Failed to append frame to buffer: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(source = %self.capture.stream_id(), "capture failed: {}", e);
                }
            }
            sleep(self.capture.interval()).await;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Capture that produces nothing — lets us exercise the lifecycle without
    /// touching the OS.
    struct NoopCapture {
        output_dir: String,
    }

    #[async_trait]
    impl Capture for NoopCapture {
        type Config = String;
        fn from_config(output_dir: String) -> Result<Self, LifelogError> {
            Ok(Self { output_dir })
        }
        fn output_dir(&self) -> &str {
            &self.output_dir
        }
        fn stream_id(&self) -> &str {
            "noop"
        }
        fn interval(&self) -> Duration {
            Duration::from_millis(10)
        }
        async fn capture(&self) -> Result<Vec<Vec<u8>>, LifelogError> {
            Ok(vec![])
        }
    }

    /// Regression test for the module-global `static RUNNING` bug: stopping one
    /// instance must not stop a sibling instance of the same source type. With
    /// the old global flag this failed (all instances shared one bool).
    #[tokio::test]
    async fn stopping_one_instance_leaves_siblings_running() {
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();

        let a =
            PollingSource::<NoopCapture>::new(dir_a.path().to_str().unwrap().to_string()).unwrap();
        let b =
            PollingSource::<NoopCapture>::new(dir_b.path().to_str().unwrap().to_string()).unwrap();

        let _ha = a.start().unwrap();
        let _hb = b.start().unwrap();
        assert!(a.is_running());
        assert!(b.is_running());

        // Stop A via a clone that shares A's running flag.
        let mut a_stop = a.clone();
        a_stop.stop().await.unwrap();

        assert!(!a.is_running(), "instance A should be stopped");
        assert!(
            b.is_running(),
            "sibling instance B must keep running (per-instance flag, not module-global)"
        );

        let mut b_stop = b.clone();
        b_stop.stop().await.unwrap();
    }
}
