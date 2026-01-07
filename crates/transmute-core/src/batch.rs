use futures::stream::{self, StreamExt};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use transmute_common::{Error, MediaFormat, Result};

/// Single conversion job in a batch
#[derive(Debug, Clone)]
pub struct BatchJob {
    pub input: PathBuf,
    pub output_format: MediaFormat,
    pub output_path: Option<PathBuf>,
}

/// Progress tracking for batch operations
#[derive(Debug, Clone)]
pub struct BatchProgress {
    pub completed: usize,
    pub total: usize,
    pub current_file: Option<PathBuf>,
    pub failed: Vec<(PathBuf, String)>, // (file, error_message)
}

impl BatchProgress {
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f32 / self.total as f32) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.completed == self.total
    }
}

/// Async batch processor with progress tracking
pub struct BatchProcessor {
    /// Maximum concurrent conversions
    concurrency: usize,
}

impl BatchProcessor {
    pub fn new(concurrency: usize) -> Self {
        let concurrency = if concurrency == 0 {
            num_cpus::get()
        } else {
            concurrency
        };

        tracing::info!(
            "BatchProcessor initialized with concurrency={}",
            concurrency
        );
        Self { concurrency }
    }

    /// Process batch with progress updates via channel
    pub async fn process_batch(
        &self,
        jobs: Vec<BatchJob>,
        progress_tx: mpsc::UnboundedSender<BatchProgress>,
    ) -> Result<Vec<Result<PathBuf>>> {
        let total = jobs.len();
        tracing::info!("Starting batch processing: {} jobs", total);

        let progress = Arc::new(Mutex::new(BatchProgress {
            completed: 0,
            total,
            current_file: None,
            failed: Vec::new(),
        }));

        // Send initial progress
        let initial_progress = progress.lock().await.clone();
        let _ = progress_tx.send(initial_progress);

        // Process jobs concurrently with limit
        let results: Vec<Result<PathBuf>> = stream::iter(jobs)
            .map(|job| {
                let progress = Arc::clone(&progress);
                let progress_tx = progress_tx.clone();

                async move {
                    // Update current file
                    {
                        let mut p = progress.lock().await;
                        p.current_file = Some(job.input.clone());
                        let _ = progress_tx.send(p.clone());
                    }

                    // Perform conversion (spawn blocking for CPU work)
                    let result = tokio::task::spawn_blocking({
                        let job = job.clone();
                        move || Self::process_single_job(job)
                    })
                    .await
                    .map_err(|e| Error::ConversionError(format!("Task join error: {}", e)))?;

                    // Update progress
                    {
                        let mut p = progress.lock().await;
                        p.completed += 1;

                        if let Err(ref e) = result {
                            p.failed.push((job.input.clone(), e.to_string()));
                        }

                        let _ = progress_tx.send(p.clone());
                    }

                    result
                }
            })
            .buffer_unordered(self.concurrency) // Key: limit concurrent tasks
            .collect()
            .await;

        tracing::info!(
            "Batch processing complete: {}/{} succeeded",
            results.iter().filter(|r| r.is_ok()).count(),
            total
        );

        Ok(results)
    }

    /// Process single job (synchronous, called in spawn_blocking)
    fn process_single_job(job: BatchJob) -> Result<PathBuf> {
        use crate::converter::Converter;

        let converter = Converter::new()?;
        converter.convert_image(&job.input, job.output_format, job.output_path)
    }

    /// Convenience method: process batch and wait for completion
    pub async fn process_batch_sync(&self, jobs: Vec<BatchJob>) -> Result<Vec<Result<PathBuf>>> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn processing task
        let processor_handle = {
            let concurrency = self.concurrency;
            let jobs = jobs.clone();
            tokio::spawn(async move {
                let processor = BatchProcessor { concurrency };
                processor.process_batch(jobs, tx).await
            })
        };

        // Consume progress updates (in real app, would update UI)
        tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                tracing::debug!(
                    "Progress: {}/{} ({:.1}%)",
                    progress.completed,
                    progress.total,
                    progress.percentage()
                );
            }
        });

        // Wait for results
        processor_handle
            .await
            .map_err(|e| Error::ConversionError(format!("Batch processing failed: {}", e)))?
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(0) // Auto-detect CPU count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_batch_processing() {
        let temp_dir = TempDir::new().unwrap();

        // Create test images
        let mut jobs = Vec::new();
        for i in 0..5 {
            let input_path = temp_dir.path().join(format!("test{}.png", i));
            let img = DynamicImage::new_rgb8(100, 100);
            img.save(&input_path).unwrap();

            jobs.push(BatchJob {
                input: input_path,
                output_format: MediaFormat::Jpeg,
                output_path: Some(temp_dir.path().to_path_buf()),
            });
        }

        let processor = BatchProcessor::new(2); // 2 concurrent jobs
        let results = processor.process_batch_sync(jobs).await.unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 5);
    }

    #[tokio::test]
    async fn test_progress_tracking() {
        let temp_dir = TempDir::new().unwrap();

        let mut jobs = Vec::new();
        for i in 0..3 {
            let input_path = temp_dir.path().join(format!("test{}.png", i));
            let img = DynamicImage::new_rgb8(50, 50);
            img.save(&input_path).unwrap();

            jobs.push(BatchJob {
                input: input_path,
                output_format: MediaFormat::Jpeg,
                output_path: Some(temp_dir.path().to_path_buf()),
            });
        }

        let (tx, mut rx) = mpsc::unbounded_channel();
        let processor = BatchProcessor::new(1);

        // Spawn processor
        tokio::spawn(async move { processor.process_batch(jobs, tx).await });

        // Collect progress updates
        let mut final_progress = None;
        while let Some(progress) = rx.recv().await {
            final_progress = Some(progress);
        }

        // Verify final state: all 3 jobs completed
        let final_progress = final_progress.unwrap();
        assert_eq!(final_progress.completed, 3);
        assert_eq!(final_progress.total, 3);
        assert!(final_progress.is_complete());
    }
}
