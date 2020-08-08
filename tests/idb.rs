use js_sys::Uint8Array;
use wasm_bindgen_test::{ wasm_bindgen_test, wasm_bindgen_test_configure, console_log };
use indexed_kv::IndexedKv;


wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_idb() {
    let window = web_sys::window().unwrap();

    console_log!("start");

    let ikv = IndexedKv::open(&window, "testkv")
        .await
        .unwrap();

    console_log!("open ok");

    ikv.put(b"key", Uint8Array::from("val".as_bytes()))
        .await
        .unwrap();

    console_log!("put ok");

    let val = ikv.get(b"key")
        .await
        .unwrap()
        .to_vec();

    console_log!("get: {:?}", String::from_utf8_lossy(&val));

    assert_eq!(val, b"val");

    console_log!("get ok");

    ikv.del(b"key")
        .await
        .unwrap();

    let val = ikv.get(b"key")
        .await
        .unwrap();

    assert_eq!(val.length(), 0);
}

#[wasm_bindgen_test]
async fn test_idb_get_empty() {
    let window = web_sys::window().unwrap();
    let ikv = IndexedKv::open(&window, "testkv")
        .await
        .unwrap();

    let val = ikv.get(b"new key")
        .await
        .unwrap();

    assert_eq!(val.length(), 0);
}

#[wasm_bindgen_test]
async fn test_idb_put_put() {
    let window = web_sys::window().unwrap();
    let ikv = IndexedKv::open(&window, "testkv")
        .await
        .unwrap();

    ikv.put(b"key2", "val0".as_bytes().into())
        .await
        .unwrap();

    ikv.put(b"key2", "val1".as_bytes().into())
        .await
        .unwrap();

    let val = ikv.get(b"key2")
        .await
        .unwrap()
        .to_vec();

    assert_eq!(val, b"val1");
}

#[wasm_bindgen_test]
async fn test_idb_del_empty() {
    let window = web_sys::window().unwrap();
    let ikv = IndexedKv::open(&window, "testkv")
        .await
        .unwrap();

    ikv.del(b"key3")
        .await
        .unwrap();
}

#[wasm_bindgen_test]
async fn test_idb_find() {
    let window = web_sys::window().unwrap();
    let ikv = IndexedKv::open(&window, "testkv")
        .await
        .unwrap();

    let iter = ikv.find(b"")
        .await
        .unwrap();
}
