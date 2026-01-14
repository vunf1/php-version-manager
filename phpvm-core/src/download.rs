use crate::config;
use crate::logging;
use anyhow::Context;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use futures::StreamExt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Instant, Duration};

pub struct Downloader {
    client: reqwest::Client,
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new() -> anyhow::Result<Self> {
        let cache_dir = config::get_base_directory().join("cache");
        fs::create_dir_all(&cache_dir)?;

        Ok(Downloader {
            client: reqwest::Client::builder()
                .user_agent("phpvm/0.1.0")
                .build()?,
            cache_dir,
        })
    }

    pub async fn download_file(
        &self,
        url: &str,
        expected_checksum: Option<&str>,
        mut progress_callback: Option<Box<dyn FnMut(u64, u64, f64) + Send + Sync>>,
    ) -> anyhow::Result<PathBuf> {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let url_hash = format!("{:x}", hasher.finish());
        let cache_path = self.cache_dir.join(&url_hash);

        if cache_path.exists() {
            if let Some(checksum) = expected_checksum {
                let actual = self.calculate_checksum(&cache_path).await?;
                if actual == checksum {
                    // File is cached - emit progress event with full download info
                    if let Some(callback) = &mut progress_callback {
                        let file_size = fs::metadata(&cache_path)
                            .map(|m| m.len())
                            .unwrap_or(0);
                        // Emit cached file info: downloaded = total (100% complete)
                        callback(file_size, file_size, 0.0); // Speed is 0 for cached files
                    }
                    return Ok(cache_path);
                }
            } else {
                // File is cached - emit progress event with full download info
                if let Some(callback) = &mut progress_callback {
                    let file_size = fs::metadata(&cache_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    // Emit cached file info: downloaded = total (100% complete)
                    callback(file_size, file_size, 0.0); // Speed is 0 for cached files
                }
                return Ok(cache_path);
            }
        }

        tracing::info!("Downloading from: {}", url);
        logging::log_message("INFO", &format!("Downloading from: {}", url));
        
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to send request to: {}", url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP error {} when downloading from: {}", status, url);
        }

        let total_size = response.content_length();
        logging::log_message("INFO", &format!("Download size: {} bytes", total_size.unwrap_or(0)));
        
        let mut file = fs::File::create(&cache_path)
            .with_context(|| format!("Failed to create cache file: {:?}", cache_path))?;
        let mut stream = response.bytes_stream();

        let mut downloaded: u64 = 0;
        let total = total_size.unwrap_or(0);
        let start_time = Instant::now();
        let mut last_update = Instant::now();
        let mut last_downloaded = 0u64;
        const UPDATE_INTERVAL: Duration = Duration::from_millis(100); // Update every 100ms

        while let Some(item) = stream.next().await {
            let chunk = item.with_context(|| "Error while downloading chunk")?;
            file.write_all(&chunk)
                .with_context(|| "Failed to write chunk to file")?;
            downloaded += chunk.len() as u64;

            // Calculate speed and emit progress
            let now = Instant::now();
            if let Some(callback) = &mut progress_callback {
                if now.duration_since(last_update) >= UPDATE_INTERVAL || downloaded == total {
                    let elapsed = now.duration_since(start_time);
                    let speed_bytes_per_sec = if elapsed.as_secs() > 0 {
                        downloaded / elapsed.as_secs()
                    } else {
                        let chunk_size = downloaded.saturating_sub(last_downloaded);
                        let time_elapsed = now.duration_since(last_update).as_secs_f64();
                        if time_elapsed > 0.0 {
                            (chunk_size as f64 / time_elapsed) as u64
                        } else {
                            0
                        }
                    };
                    let speed_mbps = (speed_bytes_per_sec as f64) / (1024.0 * 1024.0);
                    callback(downloaded, total, speed_mbps);
                    last_update = now;
                    last_downloaded = downloaded;
                }
            } else if let Some(total) = total_size {
                let percent = (downloaded * 100) / total;
                tracing::debug!("Download progress: {}%", percent);
            }
        }
        
        logging::log_message("INFO", &format!("Download completed: {} bytes saved to {:?}", downloaded, cache_path));

        if let Some(checksum) = expected_checksum {
            let actual = self.calculate_checksum(&cache_path).await?;
            if actual != checksum {
                fs::remove_file(&cache_path)?;
                anyhow::bail!(
                    "Checksum mismatch: expected {}, got {}",
                    checksum,
                    actual
                );
            }
        }

        Ok(cache_path)
    }

    async fn calculate_checksum(&self, path: &PathBuf) -> anyhow::Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
}
