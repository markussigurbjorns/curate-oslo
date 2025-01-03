use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    routing::post,
    Router,
};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let frontend = async {
        // Serve static files
        let not_found_service = Router::new().route(
            "/*path",
            get(handler_404), // convert handler to a GET route
        );

        let listener = tokio::net::TcpListener::bind("0.0.0.0:6868").await.unwrap();
        let app = Router::new().nest_service(
            "/",
            ServeDir::new("public").not_found_service(not_found_service),
            //ServeDir::new("wasm-frontend/pkg").not_found_service(not_found_service),
        );
        axum::serve(listener, app).await.unwrap();
    };

    let backend = async {
        // Directory to create
        let storage_dir = Arc::new(Mutex::new("uploads".to_string()));
        fs::create_dir_all(&*storage_dir.lock().await).unwrap();

        // Define CORS layer
        // Todo: Only allow curate oslo domain in the future
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

        // Set up routes
        let app = Router::new()
            .route(
                "/upload",
                post(upload_file_handler).layer(RequestBodyLimitLayer::new(300_000_000)),
            )
            .route("/files", get(list_files_handler))
            .route("/download/:filename", get(download_file_handler))
            .route("/play/:filename", get(play_audio_file_handler))
            .layer(DefaultBodyLimit::disable())
            .with_state(storage_dir.clone())
            .layer(cors);

        //run server
        let listener = tokio::net::TcpListener::bind("0.0.0.0:6969").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    };

    println!("Listening on 0.0.0.0:6868");
    tokio::join!(frontend, backend);
}
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND,
            Html("<!DOCTYPE html><html><head><title>404</title></head><body><h1>nothing to see here</h1></body></html>"))
}

async fn upload_file_handler(
    State(storage_dir): State<Arc<Mutex<String>>>,
    mut multipart: Multipart,
) {
    let storage_dir = storage_dir.lock().await.clone();

    let mut alias: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "alias" => {
                alias = Some(field.text().await.unwrap_or_default());
            }
            "file" => {
                if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                    filename = Some(fname);
                }
                let mut bytes = Vec::new();
                while let Some(chunk) = field.chunk().await.unwrap() {
                    bytes.extend_from_slice(&chunk);
                }
                file_bytes = Some(bytes);
            }
            _ => {}
        }
    }
    if let (Some(bytes), Some(original_name)) = (file_bytes, filename) {
        let new_name = if let Some(ref alias_str) = alias {
            let new_alias: String = alias_str
                .chars()
                .map(|c| if c.is_whitespace() { '_' } else { c })
                .collect();
            format!("{}_{}", new_alias, original_name)
        } else {
            original_name
        };

        let file_path = PathBuf::from(&storage_dir).join(new_name);
        let mut file = File::create(&file_path).expect("Failed to create file on the server");
        file.write_all(&bytes)
            .expect("Failed to write file to disk");
    }
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
}

async fn list_files_handler(State(storage_dir): State<Arc<Mutex<String>>>) -> Json<Vec<FileInfo>> {
    let storage_dir = storage_dir.lock().await.clone();
    let dir_path = PathBuf::from(&storage_dir);

    let mut file_list = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir(dir_path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_name) = entry.file_name().into_string() {
                file_list.push(FileInfo { name: file_name });
            }
        }
    }
    Json(file_list)
}

async fn download_file_handler(
    State(storage_dir): State<Arc<Mutex<String>>>,
    Path(filename): Path<String>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let storage_dir = storage_dir.lock().await.clone();
    let file_path = PathBuf::from(&storage_dir).join(&filename);

    if file_path.exists() {
        let file = tokio::fs::File::open(file_path)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        let stream = tokio_util::io::ReaderStream::new(file);
        let body = axum::body::Body::from_stream(stream);

        Ok(axum::response::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            )
            .body(body.into())
            .unwrap())
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn play_audio_file_handler(
    State(storage_dir): State<Arc<Mutex<String>>>,
    Path(filename): Path<String>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let storage_dir = storage_dir.lock().await.clone();
    let file_path = PathBuf::from(&storage_dir).join(&filename);

    //add content-length to the headers
    //and handle range requsests
    if file_path.exists() {
        let file = tokio::fs::File::open(file_path.clone())
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        let content_length = file.metadata().await.unwrap().len();
        let stream = tokio_util::io::ReaderStream::new(file);
        let body = axum::body::Body::from_stream(stream);

        let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("mp3") => "audio/mpeg",
            Some("wav") => "audio/wav",
            Some("ogg") => "audio/ogg",
            _ => "application/octet-stream",
        };

        Ok(axum::response::Response::builder()
            .header("content-type", content_type)
            .header("content-length", content_length)
            .body(body.into())
            .unwrap())
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}
