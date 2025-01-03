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

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    if let Some(form) = document.get_element_by_id("upload-form") {

        let window_clone = window.clone();
        let document_clone = document.clone();

        let closure = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();

            let file_input = document_clone
                .get_element_by_id("audio-file")
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

            let alias_input = document_clone
                .get_element_by_id("alias-input")
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap();

            let alias = alias_input.value();

            let window_async = window_clone.clone();
            let document_async = document_clone.clone();

            spawn_local(async move {
                let form_data = FormData::new().unwrap();
                form_data.append_with_str("alias", &alias).unwrap();
                form_data.append_with_blob("file", &file).unwrap();

                let mut opts = RequestInit::new();
                opts.set_method("POST");
                opts.body(Some(&form_data));

                log("before calling the url");
                
                let url = "https://curateoslo.com:7000/upload";
                let resp_promise = window_async.fetch_with_str_and_init(url, &opts);
                log("after creating promise");

                let resp: Response = match JsFuture::from(resp_promise).await {
                    Ok(r) => r.dyn_into().unwrap(),
                    Err(e) => {
                        set_status_text(
                            &document_async,
                            "upload-status",
                            &format!("Error: {:?}", e),
                        );
                        return;
                    }
                };
                log("resovling the promise");
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
        form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    } else {
        web_sys::console::error_1(&"Could not find form with id 'upload-form'".into());
    }

        // Set up the "change" event on the input
    if let Some(audio_file_input) = document.get_element_by_id("audio-file") {
        let document_clone = document.clone();

        let on_change = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let input = event
                .target()
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap();

            // Get the first file (if any)
            if let Some(file_list) = input.files() {
                if let Some(file) = file_list.get(0) {
                    // Get the file name
                    let file_name = file.name();
                    // Show it in #selected-file-name
                    if let Some(selected_file_div) = document_clone.get_element_by_id("selected-file-name") {
                        selected_file_div.set_inner_html(&file_name);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        audio_file_input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())?;
        on_change.forget();
    }

    Ok(())
}

#[wasm_bindgen]
pub fn add_audio_element(track: &str) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("no global `window` exists"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("should have a document on window"))?;

    let audio_el = document.create_element("audio")?;
    audio_el.set_attribute("controls", "")?;

    let connection_string = format!("https://curateoslo.com:7000/play/{}", track);
    let source_el = document.create_element("source")?;
    source_el.set_attribute("src", &connection_string)?;
    source_el.set_attribute("type", "audio/mpeg")?;

    audio_el.append_child(&source_el)?;

    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("document should have a body"))?;
    body.append_child(&audio_el)?;

    Ok(())
}
