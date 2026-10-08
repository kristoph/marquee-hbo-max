use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Clone)]
pub struct LazyImage {
    uri: String,
    ready: Arc<AtomicBool>,
}

impl LazyImage {
    pub fn cached_at(path: &Path) -> Self {
        Self { uri: format!("file://{}", path.display()), ready: Arc::new(AtomicBool::new(path.exists())) }
    }

    pub fn uri(&self) -> Option<&str> {
        self.is_ready().then_some(self.uri.as_str())
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    pub fn download(&self, source: &str, width: u32) -> Download {
        Download { source: source.to_string(), width, ready: self.ready.clone() }
    }
}

#[derive(Clone)]
pub struct SizedImage {
    pub image: LazyImage,
    pub aspect: f32,
}

impl SizedImage {
    pub fn uri(&self) -> Option<&str> {
        self.image.uri()
    }
}

pub struct Download {
    pub source: String,
    pub width: u32,
    ready: Arc<AtomicBool>,
}

impl Download {
    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::Relaxed);
    }
}
