use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    dir: PathBuf,
}

pub async fn process_http_serve(dir: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on {}", dir, addr);

    let state = HttpServeState { dir: dir.clone() };
    let dir_service = ServeDir::new(dir)
        .append_index_html_on_directories(false)
        .precompressed_gzip()
        .precompressed_br()
        .precompressed_deflate()
        .precompressed_zstd();

    let router = Router::new()
        .route("/*path", get(file_handler))
        //将/tower目录映射到ServeDir
        .nest_service("/tower", dir_service)
        //每次调用handler时都会clone，因此这里使用Arc减少性能消耗
        .with_state(Arc::new(state));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

//pub struct State<S>(pub S); 这里是pattern match写法，直接获取里面的S，否则如果是state: State<Arc<HttpServeState>>，那么要通过state.0去获取里面的内容
async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    println!("dir:{:?}, path:{:?}", &state.dir, path);
    let p = std::path::Path::new(&state.dir).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        (
            StatusCode::NOT_FOUND,
            format!("File {} not found", p.display()),
        )
    } else if p.is_dir() {
        match tokio::fs::read_dir(p).await {
            Ok(mut dir) => {
                let mut dirs: Vec<String> = Vec::new();
                loop {
                    match dir.next_entry().await {
                        Ok(Some(entry)) => {
                            if let Err(e) = entry.file_name().into_string() {
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("Get filename oserror: {:?}", e),
                                );
                            }
                            dirs.push(entry.file_name().into_string().unwrap());
                        }
                        Ok(None) => {
                            break;
                        }
                        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    }
                }
                let dirs = dirs.join("\n");
                (StatusCode::OK, dirs.to_string())
            }
            Err(e) => {
                warn!("Error reading dir: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    } else {
        match tokio::fs::read_to_string(p).await {
            Ok(content) => {
                info!("Read {} bytes", content.len());
                (StatusCode::OK, content)
            }
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            dir: PathBuf::from("."),
        });
        let (status, content) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        assert!(content.trim().starts_with("[package]"));
    }
}
