use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,

    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(dur: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));
        let clone_shared_state = Arc::clone(&shared_state);
        // 新建一个 thread
        thread::spawn(move || {
            // 睡眠 dur 时间
            thread::sleep(dur);

            // 将 Timer shared_state修改为完成的
            // 并且会触发 wake，将执行器唤醒？
            let mut shared_state = clone_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
                //  wake的时候会把执行 executor.rs 中 wake_by_ref 。
            }
        });

        TimerFuture { shared_state }
    }
}
