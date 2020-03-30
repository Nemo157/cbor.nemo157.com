use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn parse_auto(s: &str) -> JsValue {
    let result = cbor_diag::parse_hex(s)
        .or_else(|_| cbor_diag::parse_diag(s))
        .map(|v| (v.to_hex(), v.to_diag_pretty_colored_html()))
        .map_err(|e| format!("{:?}", e));
    JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn parse_hex(hex: &str) -> JsValue {
    let result = cbor_diag::parse_hex(hex)
        .map(|v| (v.to_hex(), v.to_diag_pretty_colored_html()))
        .map_err(|e| format!("{:?}", e));
    JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn parse_diag(diag: &str) -> JsValue {
    let result = cbor_diag::parse_diag(diag)
        .map(|v| (v.to_hex(), v.to_diag_pretty_colored_html()))
        .map_err(|e| format!("{:?}", e));
    JsValue::from_serde(&result).unwrap()
}
