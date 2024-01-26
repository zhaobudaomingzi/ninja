use crate::{
    arkose::{self, Type},
    context::WORKER_DIR,
    homedir::home_dir,
    info, warn,
};
use anyhow::anyhow;
use hotwatch::{Event, EventKind, Hotwatch};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock, RwLock,
    },
};
use tokio::fs::ReadDir;

use crate::arkose::crypto;
use crate::urldecoding;
use anyhow::Result;
use base64::Engine;
use moka::sync::Cache;
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;

pub static HAR: OnceLock<RwLock<HashMap<arkose::Type, HarProvider>>> = OnceLock::new();

struct HarPath {
    dir: PathBuf,
    filepath: Option<PathBuf>,
}

#[derive(Debug)]
pub struct HarProvider {
    /// HAR dir path
    dir: PathBuf,
    /// File Hotwatch
    hotwatch: Hotwatch,
    /// HAR file pool
    pool: (AtomicUsize, Vec<String>),
}

impl HarProvider {
    pub fn new(
        _type: arkose::Type,
        dir_path: Option<&PathBuf>,
        default_dir_name: &str,
    ) -> HarProvider {
        let dir = dir_path.cloned().unwrap_or(
            home_dir()
                .expect("Failed to get home directory")
                .join(WORKER_DIR)
                .join(default_dir_name),
        );

        init_directory(&dir);

        let mut pool = Vec::new();
        Self::init(&dir, &mut pool);

        HarProvider {
            pool: (AtomicUsize::new(0), pool),
            hotwatch: watch_har_dir(_type, &dir),
            dir,
        }
    }

    fn init(dir_path: impl AsRef<Path>, pool: &mut Vec<String>) {
        std::fs::read_dir(dir_path.as_ref())
            .expect("Failed to read har directory")
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|file_path| {
                file_path
                    .extension()
                    .map(|ext| ext == "har")
                    .unwrap_or(false)
            })
            .for_each(|file_path| {
                if let Some(file_name) = file_path.file_stem() {
                    pool.push(format!("{}.har", file_name.to_string_lossy()));
                }
            });
    }

    fn reset_pool(&mut self) {
        self.pool.1.clear();
        Self::init(&self.dir, &mut self.pool.1)
    }

    fn pool(&self) -> HarPath {
        let mut har_path = HarPath {
            dir: self.dir.clone(),
            filepath: None,
        };

        if self.pool.1.is_empty() {
            return har_path;
        }

        let len = self.pool.1.len();
        let mut old = self.pool.0.load(Ordering::Relaxed);
        let mut new;
        loop {
            new = (old + 1) % len;
            match self
                .pool
                .0
                .compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(x) => old = x,
            }
        }

        har_path.filepath = Some(self.dir.join(&self.pool.1[new]));
        har_path
    }
}

fn init_directory(path: impl AsRef<Path>) {
    let path = path.as_ref();

    if !path.exists() {
        info!("Create default HAR directory: {}", path.display());
        std::fs::create_dir_all(&path).expect("Failed to create har directory");
    }
}

fn watch_har_dir(_type: arkose::Type, path: impl AsRef<Path>) -> Hotwatch {
    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    hotwatch
        .watch(path.as_ref().display().to_string(), {
            let _type = _type;
            let watch_path = path.as_ref().display().to_string();
            info!("Start watching HAR directory: {}", watch_path);
            move |event: Event| match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    event.paths.iter().for_each(|path| {
                        info!(
                            "HAR directory: {watch_path} changes observed: {}",
                            path.display()
                        );
                        let lock = HAR.get().unwrap();
                        let mut har_map = lock.write().expect("Failed to get har map");
                        if let Some(har) = har_map.get_mut(&_type) {
                            // clear cache
                            if let Some(path_str) = path.as_path().to_str() {
                                get_or_init_cache().remove(path_str);
                                har.reset_pool();
                            }
                        }
                    });
                }
                _ => {}
            }
        })
        .expect("failed to watch file!");
    hotwatch
}

impl Drop for HarProvider {
    fn drop(&mut self) {
        if let Some(err) = self.hotwatch.unwatch(self.dir.as_path()).err() {
            warn!("hotwatch stop error: {err}")
        }
    }
}

// Entry cache
static CACHE: OnceLock<Cache<String, RequestEntry>> = OnceLock::new();

// Get or init cache
fn get_or_init_cache() -> &'static Cache<String, RequestEntry> {
    CACHE.get_or_init(|| Cache::new(u64::MAX))
}

// Arkose request entry
#[derive(Clone)]
pub struct RequestEntry {
    pub typed: Type,
    pub url: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub body: String,
    pub bx: String,
    pub bv: String,
}

fn get_har_path(_type: &Type) -> anyhow::Result<HarPath> {
    let lock = HAR
        .get()
        .map(|s| s.read().ok())
        .flatten()
        .ok_or_else(|| anyhow!("Failed to get har lock"))?;
    lock.get(_type)
        .map(|h| h.pool())
        .ok_or_else(|| anyhow!("Failed to get har pool"))
}

// valid har data
#[inline]
pub fn valid(s: &[u8]) -> anyhow::Result<RequestEntry> {
    let har = serde_json::from_slice::<Har>(&s)?;
    parse(har)
}

/// Get entry
#[inline]
pub fn get_entry(_type: &arkose::Type) -> anyhow::Result<RequestEntry> {
    let path = get_har_path(_type)?;
    if let Some(filepath) = path.filepath {
        parse_from_file(filepath)
    } else {
        anyhow::bail!("Failed to get har file path")
    }
}

/// Read dir
pub async fn read_dir(_type: &Type) -> Result<ReadDir> {
    let path = get_har_path(_type)?;
    Ok(tokio::fs::read_dir(path.dir).await?)
}

/// Write entry to file
#[inline]
pub async fn write_file(
    _type: &arkose::Type,
    filename: &str,
    data: impl AsRef<[u8]>,
) -> Result<()> {
    let filepath = get_har_path(_type)?.dir.join(filename);
    // only accept har file
    check_file_extension(&filepath).map_err(|s| anyhow!(s))?;
    Ok(tokio::fs::write(filepath, data).await?)
}

/// Rename file
pub async fn rename_file(_type: &Type, filename: &str, new_filename: &str) -> Result<()> {
    let dir = get_har_path(_type)?.dir;
    let old_file = PathBuf::from(&dir).join(filename);
    let new_file = PathBuf::from(&dir).join(new_filename);
    // only accept har file
    check_file_extension(&new_file).map_err(|s| anyhow!(s))?;
    Ok(tokio::fs::rename(old_file, new_file).await?)
}

/// Delete file
pub async fn delete_file(_type: &Type, filename: &str) -> Result<()> {
    // get the file path
    let filepath = get_har_path(_type)?.dir.join(filename);
    // only accept har file
    check_file_extension(&filepath).map_err(|s| anyhow!(s))?;
    Ok(tokio::fs::remove_file(filepath).await?)
}

fn check_file_extension(file: &PathBuf) -> Result<(), &'static str> {
    if let Some(ext) = file.extension() {
        if ext != "har" {
            return Err("Your file has been failed to rename: invalid file extension>");
        }
    }
    Ok(())
}

/// Parse file
#[inline]
fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<RequestEntry> {
    // Check if the path is a file
    path.as_ref()
        .is_file()
        .then(|| ())
        .ok_or_else(|| anyhow!("{} not a file", path.as_ref().display()))?;

    // Get the cache
    let cache = get_or_init_cache();

    // Get the key from the path
    let key = format!("{}", path.as_ref().display());

    // Try to get the value from the cache
    let result = cache.try_get_with(key, || {
        let bytes = std::fs::read(path)?;
        let har = serde_json::from_slice::<Har>(&bytes)?;
        drop(bytes);
        parse(har)
    });

    match result {
        Ok(value) => Ok(value),
        Err(err) => anyhow::bail!(err),
    }
}

#[inline]
fn parse(har: Har) -> Result<RequestEntry> {
    if let Some(entry) = har
        .log
        .entries
        .into_iter()
        .find(|e| e.request.url.contains("fc/gt2/public_key"))
    {
        // Check if the entry has a started date time
        if entry.started_date_time.is_empty() {
            anyhow::bail!("Invalid HAR file");
        }

        // Get the public key
        let pk = entry
            .request
            .url
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("Invalid HAR file"))?;

        let typed = Type::from_pk(pk)?;

        let url = format!("{}/fc/gt2/public_key/{}", typed.origin_url(), typed.pk());

        // Request started date time
        let started_date_time = time::OffsetDateTime::parse(&entry.started_date_time, &Rfc3339)?;

        let bt = started_date_time.unix_timestamp();
        let bw = bt - (bt % 21600);
        let mut bv = String::new();

        if let Some(data) = entry.request.post_data {
            let headers = entry.request.headers;

            if let Some(h) = headers
                .iter()
                .find(|h| h.name.eq_ignore_ascii_case("user-agent"))
            {
                bv.push_str(&h.value);
            }

            if let Some(bda_param) = data
                .params
                .unwrap_or_default()
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case("bda"))
            {
                let cow = urldecoding::decode(&bda_param.value)?;
                let bda = base64::engine::general_purpose::STANDARD.decode(cow.into_owned())?;
                let bx = crypto::decrypt(bda, &format!("{bv}{bw}"))?;

                let entry = RequestEntry {
                    typed,
                    url,
                    method: entry.request.method,
                    headers: headers
                        .into_iter()
                        .filter(|h| {
                            let name = &h.name;
                            !name.starts_with(":")
                                && !name.eq_ignore_ascii_case("content-length")
                                && !name.eq_ignore_ascii_case("connection")
                        })
                        .collect::<Vec<Header>>(),
                    body: data
                        .text
                        .unwrap_or_default()
                        .split("&")
                        .into_iter()
                        .filter(|s| {
                            !s.contains("bda")
                                && !s.contains("rnd")
                                && !s.contains("data[blob]")
                                && !s.contains("capi_version")
                        })
                        .collect::<Vec<&str>>()
                        .join("&"),
                    bx,
                    bv,
                };
                return Ok(entry);
            }
        }
    }

    anyhow::bail!("Unable to find har related request entry")
}

#[derive(Debug, Deserialize)]
struct Har {
    log: Log,
}

#[derive(Debug, Deserialize)]
struct Log {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    #[serde(rename = "request")]
    request: Request,
    #[serde(rename = "startedDateTime")]
    started_date_time: String,
}

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
    url: String,
    headers: Vec<Header>,
    #[serde(rename = "postData")]
    post_data: Option<PostData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
struct PostData {
    text: Option<String>,
    params: Option<Vec<Param>>,
}

#[derive(Debug, Deserialize)]
pub struct Param {
    pub name: String,
    pub value: String,
}
