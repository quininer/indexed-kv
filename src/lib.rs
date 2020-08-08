mod error;
mod oneshot;

use error::JsResult;
use wasm_bindgen::{ JsCast, JsValue };
use wasm_bindgen::closure::Closure;
use js_sys::Uint8Array;
use web_sys::{ Window, IdbDatabase, IdbTransactionMode };


pub struct IndexedKv {
    db: IdbDatabase
}

pub struct Iter<'a> {
    tr: &'a ()
}

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
        let idb = window.indexed_db()?
            .ok_or("indexed db not available")?;
        let req = idb.open(name)?;

        if await_req!(req) {
            Ok(IndexedKv {
                db: req.result()?.into()
            })
        } else {
            Err("indexed db open failed".into())
        }
    }

    pub async fn get(&self, key: &[u8]) -> JsResult<Vec<u8>> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readonly)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.get(Uint8Array::from(key).as_ref())?;

        if await_req!(req) {
            let val = req.result()?;
            let val = val
                .dyn_ref::<Uint8Array>()
                .ok_or("Unexpected value type")?;

            Ok(val.to_vec())
        } else {
            Err("db get failed".into())
        }
    }

    pub async fn put(&self, key: &[u8], val: &[u8]) -> JsResult<()> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readwrite)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.put_with_key(
            Uint8Array::from(key).as_ref(),
            Uint8Array::from(val).as_ref()
        )?;

        if await_req!(req) {
            Ok(())
        } else {
            Err("db put failed".into())
        }
    }

    pub async fn del(&self, key: &[u8]) -> JsResult<()> {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readwrite)?;
        let obj = tr.object_store(DEFAULT_STORE)?;
        let req = obj.delete(Uint8Array::from(key).as_ref())?;

        if await_req!(req) {
            Ok(())
        } else {
            Err("db delete failed".into())
        }
    }

    pub async fn iter(&self, prefix: &[u8]) {
        let tr = self.db.transaction_with_str_and_mode(DEFAULT_STORE, IdbTransactionMode::Readonly)?;
        let obj = tr.object_store(DEFAULT_STORE)?;

        let req = if prefix.is_empty() {
            obj.get_all()?
        } else {
            obj.get_all_with_key(Uint8Array::from(key).as_ref())?
        };

        todo!()
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
