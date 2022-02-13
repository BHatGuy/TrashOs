use conquer_once::spin::OnceCell;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use crossbeam::{atomic::AtomicCell, queue::ArrayQueue};

use crate::vga;

static WAKERS: OnceCell<ArrayQueue<Waker>> = OnceCell::uninit();
static TICKS: AtomicCell<usize> = AtomicCell::new(0);
pub fn tick() {
    TICKS.fetch_add(1);
    if let Ok(wakers) = WAKERS.try_get() {
        while let Some(waker) = wakers.pop() {
            waker.wake();
        }
    }
    
}

pub struct Sleeper {
    target_ticks: usize,
}

impl Sleeper {
    fn new(ticks: usize) -> Self {
        WAKERS.get_or_init(|| ArrayQueue::new(100));
        Self {
            target_ticks: TICKS.load() + ticks,
        }
    }
}

impl Future for Sleeper {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if TICKS.load() >= self.target_ticks {
            return Poll::Ready(());
        }
        WAKERS.try_get().expect("uninited").push(cx.waker().clone()).unwrap();
        if TICKS.load() >= self.target_ticks {
            return Poll::Ready(());
        } else {
            return Poll::Pending;
        }
    }
}

pub fn sleep(ticks: usize) -> Sleeper {
    return Sleeper::new(ticks);
}

const INDICATOR: [char; 4] = ['\\', '|', '/', '-'];
pub async fn indicator() {
    let mut index = 0;
    loop {
        index += 1;
        if index >= INDICATOR.len() {
            index = 0;
        }
        vga::WRITER.lock().write_at(INDICATOR[index] as u8, 0, 79);
        sleep(1).await;
    }
}
