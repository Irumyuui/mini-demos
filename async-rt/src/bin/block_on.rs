// use std::{sync::Arc, task::Context};

// use futures::task::{ArcWake, waker_ref};

// struct BlockOnWaker;

// impl ArcWake for BlockOnWaker {
//     fn wake_by_ref(_arc_self: &std::sync::Arc<Self>) {}
// }

// pub fn block_on<T: Send>(fut: impl Future<Output = T> + Send + 'static) -> T {
//     let waker = Arc::new(BlockOnWaker);
//     let mut fut = Box::pin(fut);

//     loop {
//         let waker = waker_ref(&waker);
//         let mut cx = Context::from_waker(&waker);

//         match fut.as_mut().poll(&mut cx) {
//             std::task::Poll::Ready(result) => return result,
//             std::task::Poll::Pending => {
//                 println!("pending...");
//             }
//         }
//     }
// }

// fn main() {
//     println!(
//         "res {}",
//         block_on(async {
//             async { println!("2") }.await;
//             1
//         })
//     )
// }

use std::task::{Context, RawWaker, RawWakerVTable, Waker};

static VTABLE: RawWakerVTable = {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }

    unsafe fn wake(_data: *const ()) {
        println!("call wake")
    }

    unsafe fn wake_by_ref(_: *const ()) {}

    unsafe fn drop(_: *const ()) {}

    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
};

fn create_block_on_waker() -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

pub fn block_on<T, Fut>(fut: Fut) -> T
where
    T: Send,
    Fut: Future<Output = T> + Send + 'static,
{
    let mut fut = Box::pin(fut);

    loop {
        let waker = create_block_on_waker();
        let mut context = Context::from_waker(&waker);

        match fut.as_mut().poll(&mut context) {
            std::task::Poll::Ready(res) => return res,
            std::task::Poll::Pending => {
                println!("pending")
            }
        }
    }
}

fn main() {
    let res = block_on(async {
        async {
            println!("2");
        }
        .await;
        1
    });

    println!("res: {:?}", res);
}
