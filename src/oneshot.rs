use std::rc::{ Rc, Weak };
use std::pin::Pin;
use std::cell::RefCell;
use std::task::{ Context, Waker, Poll };
use std::future::Future;


pub struct Sender<T>(Weak<RefCell<Inner<T>>>);

pub struct Receiver<T>(Rc<RefCell<Inner<T>>>);

struct Inner<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(RefCell::new(Inner {
        value: None,
        waker: None
    }));
    let inner2 = Rc::downgrade(&inner);

    (Sender(inner2), Receiver(inner))
}

impl<T> Sender<T> {
    pub fn send(self, item: T) -> Result<(), T> {
        if let Some(inner) = self.0.upgrade() {
            let mut inner = inner.borrow_mut();

            if inner.value.is_none() {
                inner.value = Some(item);

                if let Some(waker) = inner.waker.take() {
                    waker.wake()
                }

                Ok(())
            } else {
                Err(item)
            }
        } else {
            Err(item)
        }
    }
}

impl<T> Clone for Sender<T> {
    #[inline]
    fn clone(&self) -> Sender<T> {
        Sender(self.0.clone())
    }
}

impl<T> Future for Receiver<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.0.borrow_mut();

        if let Some(val) = inner.value.take() {
            Poll::Ready(val)
        } else {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
