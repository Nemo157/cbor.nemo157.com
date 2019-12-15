use lazy_static::lazy_static;
use log::error;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use url::Url;
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{
    Document, Element, HtmlAnchorElement, HtmlButtonElement, HtmlInputElement, HtmlTextAreaElement,
    KeyboardEvent, Window,
};

struct NoThreads<T>(T);

unsafe impl<T> Send for NoThreads<T> {}
unsafe impl<T> Sync for NoThreads<T> {}

impl<T> core::ops::Deref for NoThreads<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for NoThreads<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

#[derive(Debug)]
enum Error {
    Js(JsValue),
    Rust(Box<dyn std::error::Error>),
}

impl From<JsValue> for Error {
    fn from(val: JsValue) -> Error {
        Error::Js(val)
    }
}

impl From<cbor_diag::Error> for Error {
    fn from(val: cbor_diag::Error) -> Error {
        Error::Rust(val.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Error {
        Error::Rust(val.into())
    }
}

impl From<&'static str> for Error {
    fn from(val: &'static str) -> Error {
        Error::Rust(val.into())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Js(e) => write!(f, "{:?}", e),
            Error::Rust(e) => write!(f, "{}", e),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

lazy_static! {
    static ref WINDOW: NoThreads<Window> = web_sys::window().unwrap().into();
    static ref DOCUMENT: NoThreads<Document> = WINDOW.document().unwrap().into();
    static ref INPUT: NoThreads<HtmlTextAreaElement> = DOCUMENT
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap()
        .into();
    static ref SUBMIT: NoThreads<Element> = DOCUMENT.get_element_by_id("submit").unwrap().into();
    static ref HEX: NoThreads<Element> = DOCUMENT.get_element_by_id("hex").unwrap().into();
    static ref DIAG: NoThreads<Element> = DOCUMENT.get_element_by_id("diag").unwrap().into();
    static ref SAVE: NoThreads<Element> = DOCUMENT.get_element_by_id("save").unwrap().into();
    static ref DARK: NoThreads<Element> = DOCUMENT.get_element_by_id("dark").unwrap().into();
    static ref SAVED: NoThreads<HtmlAnchorElement> = DOCUMENT
        .get_element_by_id("saved")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap()
        .into();
    static ref COPY_BUTTON: NoThreads<HtmlButtonElement> = DOCUMENT
        .get_element_by_id("copy-button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap()
        .into();
    static ref COPIED: NoThreads<Element> = DOCUMENT.get_element_by_id("copied").unwrap().into();
    static ref AUTO_CHECKBOX: NoThreads<HtmlInputElement> = DOCUMENT
        .query_selector(r#"input[name="type"][value="auto"]"#)
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .into();
    static ref HEX_CHECKBOX: NoThreads<HtmlInputElement> = DOCUMENT
        .query_selector(r#"input[name="type"][value="hex"]"#)
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .into();
    static ref DIAG_CHECKBOX: NoThreads<HtmlInputElement> = DOCUMENT
        .query_selector(r#"input[name="type"][value="diag"]"#)
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
        .into();
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum ParseType {
    Auto,
    Hex,
    Diag,
}

fn parse(ty: ParseType, value: &str) -> Result<(String, String)> {
    match ty {
        ParseType::Auto => cbor_diag::parse_hex(value)
            .or_else(|_| cbor_diag::parse_diag(value))
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(Into::into),

        ParseType::Hex => cbor_diag::parse_hex(value)
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(Into::into),

        ParseType::Diag => cbor_diag::parse_diag(value)
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(Into::into),
    }
}

fn switch_type(ty: ParseType) {
    match ty {
        ParseType::Auto => AUTO_CHECKBOX.set_checked(true),
        ParseType::Hex => HEX_CHECKBOX.set_checked(true),
        ParseType::Diag => DIAG_CHECKBOX.set_checked(true),
    }
}

fn get_type() -> ParseType {
    if AUTO_CHECKBOX.checked() {
        ParseType::Auto
    } else if HEX_CHECKBOX.checked() {
        ParseType::Hex
    } else if DIAG_CHECKBOX.checked() {
        ParseType::Diag
    } else {
        ParseType::Auto
    }
}

fn store(setting: &str, value: &impl Serialize) {
    fn inner(setting: &str, value: &impl Serialize) -> Result<()> {
        WINDOW
            .local_storage()?
            .ok_or("no storage")?
            .set_item(setting, &serde_json::to_string(&value)?)?;
        Ok(())
    }
    if let Err(e) = inner(setting, value) {
        error!("failed storing to {}: {}", setting, e);
    }
}

fn load<T: for<'de> Deserialize<'de> + 'static>(setting: &str) -> Option<T> {
    fn inner<T: for<'de> Deserialize<'de> + 'static>(setting: &str) -> Result<T> {
        Ok(serde_json::from_str(
            &WINDOW
                .local_storage()?
                .ok_or("no storage")?
                .get_item(setting)?
                .ok_or("no item")?,
        )?)
    }
    match inner(setting) {
        Ok(v) => Some(v),
        Err(e) => {
            error!("failed loading from {}: {}", setting, e);
            None
        }
    }
}

fn try_process() -> Result<()> {
    let (hex, diag) = parse(get_type(), &INPUT.value())?;
    HEX.set_text_content(Some(&hex));
    DIAG.set_text_content(Some(&diag));
    store("type", &get_type());
    store("value", &INPUT.value());
    Ok(())
}

fn process() {
    try_process().unwrap_or_else(|e| {
        HEX.set_text_content(Some(&e.to_string()));
        DIAG.set_text_content(None);
        store("type", &get_type());
        store("value", &INPUT.value());
    });
}

fn init_listeners() {
    let on_dark_click = Closure::wrap(Box::new(|| {
        store(
            "dark",
            &DOCUMENT
                .body()
                .unwrap()
                .class_list()
                .toggle("dark")
                .unwrap(),
        );
    }) as Box<dyn Fn()>);
    DARK.add_event_listener_with_callback("click", on_dark_click.as_ref().unchecked_ref())
        .unwrap();
    on_dark_click.forget();

    DOCUMENT
        .body()
        .unwrap()
        .class_list()
        .toggle_with_force("dark", load("dark").unwrap_or(false))
        .unwrap();

    let on_submit = Closure::wrap(Box::new(process) as Box<dyn Fn()>);
    SUBMIT
        .add_event_listener_with_callback("click", on_submit.as_ref().unchecked_ref())
        .unwrap();
    on_submit.forget();

    let on_save = Closure::wrap(Box::new(|| {
        let ty = get_type();
        let mut url = Url::parse(
            &DOCUMENT
                .location()
                .unwrap()
                .to_string()
                .as_string()
                .unwrap(),
        )
        .unwrap();
        {
            // Block needed to early-drop the `query_pairs_mut` return to actually modify `url`
            url.query_pairs_mut()
                .clear()
                .append_pair("type", &ty.to_string())
                .append_pair("value", &INPUT.value());
        }
        SAVED.set_href(url.as_ref());
        SAVED.set_text("Permalink to the playground").unwrap();
        // TODO: COPY_BUTTON.style().set_property("display", "inline-block").unwrap();
    }) as Box<dyn Fn()>);
    SAVE.add_event_listener_with_callback("click", on_save.as_ref().unchecked_ref())
        .unwrap();
    on_save.forget();

    // TODO:
    //  let copyTimeout = setTimeout(() => {}, 0)
    //  copyButton.addEventListener('click', () => {
    //    if (copy(saved.href, { format: 'text/plain' })) {
    //      copied.style.display = 'inline-block'
    //      copied.style.transition = 'opacity 0.1s'
    //      copied.style.opacity = '1'
    //      clearTimeout(copyTimeout)
    //      copyTimeout = setTimeout(() => {
    //        copied.style.transition = 'opacity 5s'
    //        copied.style.opacity = '0'
    //        copyTimeout = setTimeout(() => {
    //          copied.style.display = 'none'
    //        }, 5000)
    //      }, 100)
    //    }
    //  })

    let on_input_keydown = Closure::wrap(Box::new(|e: KeyboardEvent| {
        if e.key_code() == 13 && (e.meta_key() || e.ctrl_key()) {
            process()
        }
    }) as Box<dyn Fn(KeyboardEvent)>);
    INPUT
        .add_event_listener_with_callback("keydown", on_input_keydown.as_ref().unchecked_ref())
        .unwrap();
    on_input_keydown.forget();

    let on_input_keyup = Closure::wrap(Box::new(|| {
        try_process().ok();
    }) as Box<dyn Fn()>);
    INPUT
        .add_event_listener_with_callback("keyup", on_input_keyup.as_ref().unchecked_ref())
        .unwrap();
    on_input_keyup.forget();
}

fn load_values() {
    let url = Url::parse(
        &DOCUMENT
            .location()
            .unwrap()
            .to_string()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let params: HashMap<Cow<str>, Cow<str>> = url.query_pairs().collect();

    let (ty, value) = params
        .get("type")
        .and_then(|ty| ty.parse().ok())
        .and_then(|ty| params.get("value").map(|value| (ty, value.to_owned())))
        .or_else(|| load("type").and_then(|ty| load("value").map(|value| (ty, value))))
        .unwrap_or_else(|| (ParseType::Hex, "bf6346756ef563416d7421ff".into()));

    switch_type(ty);
    INPUT.set_value(&value);
    process();
}

#[wasm_bindgen(start)]
pub fn main() {
    init_listeners();
    load_values();
}
