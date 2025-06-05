use actix_web::{web, HttpResponse, Responder};
use bloomfilter::Bloom;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::debug;

use crate::storages::Storage;

static DEFAULT_URL: OnceLock<String> = OnceLock::new();

pub struct RedirectService {
    bloom_filter: Arc<RwLock<Bloom<String>>>,
}

impl RedirectService {
    #[allow(dead_code)]
    pub async fn new(storage: web::Data<Arc<dyn Storage>>) -> Self {
        // 创建布隆过滤器，从 storage 中加载所有链接
        let links_count = storage.load_all().await.len();
        debug!("Loaded {} links from storage for bloom filter", links_count);

        let mut bloom = Bloom::new_for_fp_rate(links_count + 1000, 0.01).unwrap_or_else(|_| {
            Bloom::new_for_fp_rate(100_000, 0.01)
                .unwrap_or_else(|_| panic!("Failed to create bloom filter with initial capacity"))
        });

        // 将所有链接添加到布隆过滤器中
        for (code, _) in storage.load_all().await {
            bloom.set(&code);
        }

        Self {
            bloom_filter: Arc::new(RwLock::new(bloom)),
        }
    }

    pub async fn handle_redirect(
        path: web::Path<String>,
        storage: web::Data<Arc<dyn Storage>>,
        service: web::Data<Arc<RedirectService>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        debug!("Captured path: {}", captured_path);

        if captured_path.is_empty() {
            let default_url = DEFAULT_URL
                .get_or_init(|| {
                    env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string())
                })
                .as_str();

            return HttpResponse::TemporaryRedirect()
                .insert_header(("Location", default_url))
                .insert_header(("Cache-Control", "public, max-age=300")) // 5分钟缓存
                .finish();
        }

        // 先检查布隆过滤器
        {
            let bloom = service.bloom_filter.read().await;
            if !bloom.check(&captured_path) {
                debug!("Bloom filter check failed for: {}", captured_path);
                return HttpResponse::NotFound()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .body("Not Found");
            }
        }

        match storage.get(&captured_path).await {
            Some(link) => {
                if let Some(expires_at) = link.expires_at {
                    if expires_at < chrono::Utc::now() {
                        return HttpResponse::NotFound()
                            .insert_header(("Content-Type", "text/html; charset=utf-8"))
                            .insert_header(("Cache-Control", "public, max-age=60"))
                            .body("Not Found");
                    }
                }

                // 确保链接在布隆过滤器中
                {
                    Self::add_code_to_bloom(&service, &captured_path).await;
                }

                let storage_clone = storage.clone();
                let code_clone = captured_path.clone();
                tokio::spawn(async move {
                    let _ = storage_clone.increment_click(&code_clone).await;
                });

                debug!("Redirecting {} -> {}", captured_path, link.target);

                HttpResponse::TemporaryRedirect()
                    .insert_header(("Location", link.target))
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .finish()
            }
            None => {
                debug!("Redirect link not found: {}", captured_path);
                HttpResponse::NotFound()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .insert_header(("Cache-Control", "public, max-age=60")) // 缓存404
                    .body("Not Found")
            }
        }
    }

    pub async fn add_code_to_bloom(&self, code: &String) {
        let mut bloom = self.bloom_filter.write().await;
        bloom.set(code);
        debug!("Added code to bloom filter: {}", code);
    }
}
