use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::{routing::get, routing::post, Router};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {

    //directory to create
    let storage_dir = Arc::new(Mutex::new("uploads".to_string()));
    fs::create_dir_all(&*storage_dir.lock().await).unwrap();

    //set up routes
    let app = Router::new()
        .route(
            "/upload",
            post(upload_file_handler).layer(RequestBodyLimitLayer::new(3 * 100 * 1000 * 1000)),
        )
        .route("/download/:filename", get(download_file_handler))
        .layer(DefaultBodyLimit::disable())
        .with_state(storage_dir.clone());

    //run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6969").await.unwrap();
    println!("Listening on 0.0.0.0:6969");
    axum::serve(listener, app).await.unwrap();
}

async fn upload_file_handler(
    State(storage_dir): State<Arc<Mutex<String>>>,
    mut multipart: Multipart,
) {
    let storage_dir = storage_dir.lock().await.clone();

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        if let Some(filename) = field.file_name() {
            //handle if file allready exists
            let file_path = PathBuf::from(&storage_dir).join(filename);
            let mut file = File::create(file_path).unwrap();
            while let Some(chunk) = field.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();
            }
        }
    }
}

async fn download_file_handler(
    State(storage_dir): State<Arc<Mutex<String>>>,
    Path(filename): Path<String>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let storage_dir = storage_dir.lock().await.clone();
    let file_path = PathBuf::from(&storage_dir).join(&filename);

    if file_path.exists() {
        let file = tokio::fs::File::open(file_path).await.unwrap();
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
