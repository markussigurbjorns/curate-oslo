#![no_main]
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::Event;
use web_sys::HtmlInputElement;
use web_sys::{window, Document, File, FormData, RequestInit, Response};

fn set_status_text(document: &Document, id: &str, text: &str) {
    if let Some(elem) = document.get_element_by_id(id) {
        elem.set_text_content(Some(text));
    }
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn run() -> Result<(), JsValue> {
    // window object.
    let window = window().ok_or_else(|| JsValue::from("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from("no document"))?;

    let form = document
        .get_element_by_id("upload-form")
        .ok_or_else(|| JsValue::from("could not find form with id 'upload-form'"))?;

    let window_clone = window.clone();
    let document_clone = document.clone();

    let closure = Closure::wrap(Box::new(move |event: Event| {
        event.prevent_default();

        let file_input = document_clone
            .get_element_by_id("file-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let _status_div = document_clone.get_element_by_id("upload-status").unwrap();

        let files = file_input.files().unwrap();
        if files.length() == 0 {
            set_status_text(&document_clone, "upload-status", "Please select a file");
        }

        let file: File = files.item(0).unwrap();

        set_status_text(&document_clone, "upload-status", "Uploading...");

        let window_async = window_clone.clone();
        let document_async = document_clone.clone();

        spawn_local(async move {
            let form_data = FormData::new().unwrap();
            form_data.append_with_blob("file", &file).unwrap();

            let mut opts = RequestInit::new();
            opts.set_method("POST");
            opts.body(Some(&form_data));

            let url = "http://127.0.0.1:6969/upload";
            let resp_promise = window_async.fetch_with_str_and_init(url, &opts);
            let resp: Response = match JsFuture::from(resp_promise).await {
                Ok(r) => r.dyn_into().unwrap(),
                Err(e) => {
                    set_status_text(&document_async, "upload-status", &format!("Error: {:?}", e));
                    return;
                }
            };

            if resp.ok() {
                set_status_text(&document_async, "upload-status", "Upload successful!");
            } else {
                let status_text = resp.status_text();
                set_status_text(
                    &document_async,
                    "upload-status",
                    &format!("Upload failed: {}", status_text),
                );
            }
        });
    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();

    Ok(())
}
