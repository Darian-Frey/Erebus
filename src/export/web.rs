// Browser download trigger. Wraps the encoded PNG bytes in a Blob, asks the
// browser for an object URL, and synthesises an anchor click to drop the file
// into the user's downloads folder. Works without any server help.

use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

pub fn download_bytes(bytes: &[u8], mime: &str, filename: &str) -> anyhow::Result<()> {
    // Wrap the bytes in a Uint8Array, then a single-element Array, which is
    // what the Blob constructor accepts.
    let arr = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&arr);

    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
        .map_err(|e| anyhow::anyhow!("Blob::new failed: {e:?}"))?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| anyhow::anyhow!("createObjectURL failed: {e:?}"))?;

    let document = web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .document()
        .ok_or_else(|| anyhow::anyhow!("no document"))?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| anyhow::anyhow!("createElement failed: {e:?}"))?
        .dyn_into()
        .map_err(|_| anyhow::anyhow!("anchor cast failed"))?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // The anchor was never inserted into the DOM, so no removal needed.
    // The object URL stays alive for the lifetime of the document; revoking
    // it here would race with the download trigger on some browsers.
    let _ = url;
    Ok(())
}
