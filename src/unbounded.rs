use std::rc::{ Rc, Weak };
use std::pin::Pin;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::task::{ Context, Waker, Poll };
use std::future::Future;


pub struct Sender<T>(Weak<RefCell<Inner<T>>>);

pub struct Receiver<T>(Rc<RefCell<Inner<T>>>);

struct Inner<T> {
    queue: VecDeque<T>,
    waker: Option<Waker>,
}

pub fn channel<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(RefCell::new(Inner {
        queue: VecDeque::with_capacity(cap),
        waker: None
    }));
    let inner2 = Rc::downgrade(&inner);

    (Sender(inner2), Receiver(inner))
}

impl<T> Sender<T> {
    pub fn send(&self, item: T) -> Result<(), T> {
        if let Some(inner) = self.0.upgrade() {
            let mut inner = inner.borrow_mut();

            inner.queue.push_back(item);

            if let Some(waker) = inner.waker.take() {
                waker.wake()
            }

            Ok(())
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

impl<T> Receiver<T> {
    pub async fn next(&mut self) -> Option<T> {
        ReceiverRef(self).await
    }
}

struct ReceiverRef<'a, T>(&'a mut Receiver<T>);

impl<T> Future for ReceiverRef<'_, T> {
    type Output = Option<T>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let ReceiverRef(this) = self.get_mut();

        if Rc::weak_count(&this.0) == 0 {
            return Poll::Ready(None);
        }

        let mut inner = this.0.borrow_mut();

        if let Some(val) = inner.queue.pop_front() {
            Poll::Ready(Some(val))
        } else {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
