mod error;
mod oneshot;
mod unbounded;

use wasm_bindgen::{ JsCast, JsValue };
use wasm_bindgen::closure::Closure;
use js_sys::{ Uint8Array, ArrayBuffer };
use web_sys::{ Window, IdbDatabase, IdbTransactionMode, IdbCursorWithValue, DomException };

pub use error::{ JsResult, JsError };


pub struct IndexedKv {
    db: IdbDatabase
}

pub struct Iter {
    rx: unbounded::Receiver<Option<JsResult<Packet>>>
}

type Packet = (Vec<u8>, Uint8Array);

const DEFAULT_STORE: &str = "default";

macro_rules! await_req {
    ( $req:expr ) => {{
        let (tx, rx) = oneshot::channel::<bool>();
        let onsuccess = send_closure(tx.clone(), true);
        let onerror = send_closure(tx, false);
        $req.set_onsuccess(Some(onsuccess.unchecked_ref()));
        $req.set_onerror(Some(onerror.unchecked_ref()));
        rx.await
    }}
}

impl IndexedKv {
    pub async fn open(window: &Window, name: &str) -> JsResult<IndexedKv> {
        let builder = window.indexed_db()?
            .ok_or("indexed db not available")?;
        let req = builder.open(name)?;
        let req2 = req.clone();

        let onupgradeneeded = Closure::once_into_js(move || {
            if let Ok(val) = req2.result() {
                let db: IdbDatabase = val.into();

                if !db.object_store_names().contains(DEFAULT_STORE) {
                    db.create_object_store(DEFAULT_STORE)
                        .expect("create store failed");
                }
            }
        });
        req.set_onupgradeneeded(Some(onupgradeneeded.unchecked_ref()));

        if await_req!(req) {
            Ok(IndexedKv {
                db: req.result()?.into()
            })
        } else {
            Err(cast_dom_exception(req.error(), "indexed db open failed"))
        }
    }

    pub async fn get(&self, key: &[u8]) -> JsResult<Uint8Array> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readonly)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.get(Uint8Array::from(key).as_ref())?;

        if await_req!(req) {
            let val: ArrayBuffer = req.result()?.into();

            Ok(Uint8Array::new(val.as_ref()))
        } else {
            Err(cast_dom_exception(req.error(), "db get failed"))
        }
    }

    pub async fn put(&self, key: &[u8], val: Uint8Array) -> JsResult<()> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readwrite)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.put_with_key(val.as_ref(), Uint8Array::from(key).as_ref())?;

        if await_req!(req) {
            Ok(())
        } else {
            Err(cast_dom_exception(req.error(), "db put failed"))
        }
    }

    pub async fn del(&self, key: &[u8]) -> JsResult<()> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readwrite)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.delete(Uint8Array::from(key).as_ref())?;

        if await_req!(req) {
            Ok(())
        } else {
            Err(cast_dom_exception(req.error(), "db delete failed"))
        }
    }

    pub async fn find(&self, prefix: &[u8]) -> JsResult<Iter> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readonly)?;
        let obj = tr.object_store(DEFAULT_STORE)?;

        let req = if prefix.is_empty() {
            obj.open_cursor()?
        } else {
            obj.open_cursor_with_range(Uint8Array::from(prefix).as_ref())?
        };
        let req2 = req.clone();
        let (tx, rx) = unbounded::channel::<Option<JsResult<Packet>>>(8);

        let oncursor = Closure::wrap(Box::new(move || match req.result() {
            Ok(ret) => {
                let cursor = ret.unchecked_ref::<IdbCursorWithValue>();
                let is_continue;

                let _ = tx.send(if let (Ok(key), Ok(val)) = (cursor.key(), cursor.value()) {
                    let k: ArrayBuffer = key.into();
                    let k = Uint8Array::new(k.as_ref());
                    let v: Uint8Array = val.into();

                    is_continue = true;

                    Some(Ok((k.to_vec(), v)))
                } else {
                    is_continue = false;

                    None
                });

                if is_continue {
                    if let Err(err) = cursor.continue_() {
                        let _ = tx.send(Some(Err(err.into())));
                    }
                }
            },
            Err(err) => {
                let _ = tx.send(Some(Err(err.into())));
            }
        }) as Box<dyn FnMut()>);
        let oncursor = oncursor.into_js_value();
        req2.set_onsuccess(Some(oncursor.unchecked_ref()));

        Ok(Iter { rx })
    }
}

impl Iter {
    pub async fn next(&mut self) -> JsResult<Option<(Vec<u8>, Uint8Array)>> {
        self.rx.next().await.flatten().transpose()
    }
}

impl Drop for IndexedKv {
    fn drop(&mut self) {
        self.db.close();
    }
}

fn send_closure<T: 'static>(sender: oneshot::Sender<T>, val: T) -> JsValue {
    Closure::once_into_js(move || {
        sender.send(val)
            .ok()
            .expect("Unexpected call or Receiver canceled");
    })
}

#[cold]
fn cast_dom_exception(err: Result<Option<DomException>, JsValue>, default: &str) -> JsError {
    JsError(match err {
        Ok(Some(err)) => err.into(),
        Ok(None) => JsValue::from(default),
        Err(err) => err
    })
}
