#![allow(incomplete_features)]
#![feature(specialization)]

use std::{ptr::NonNull, sync::Mutex};

pub trait IsImplSync<T> {
    fn is_impl_sync() -> bool;
}

impl<T> IsImplSync<T> for T {
    default fn is_impl_sync() -> bool {
        false
    }
}

impl<T: Sync> IsImplSync<T> for T {
    fn is_impl_sync() -> bool {
        true
    }
}

pub trait IsImplSend<T> {
    fn is_impl_send() -> bool;
}

impl<T> IsImplSend<T> for T {
    default fn is_impl_send() -> bool {
        false
    }
}

impl<T: Send> IsImplSend<T> for T {
    fn is_impl_send() -> bool {
        true
    }
}

pub fn is_impl_send<T>() -> bool {
    <T as IsImplSend<T>>::is_impl_send()
}

pub fn is_impl_sync<T>() -> bool {
    <T as IsImplSync<T>>::is_impl_sync()
}

pub fn is_impl_send_sync<T>() -> bool {
    is_impl_send::<T>() && is_impl_sync::<T>()
}

#[macro_export]
macro_rules! print_impl_send_sync {
    ($t:ty) => {
        println!(
            "[{}]: \nis_impl_send: {}, is_impl_sync: {}, is_impl_send_sync: {}",
            stringify!($t),
            is_impl_send::<$t>(),
            is_impl_sync::<$t>(),
            is_impl_send_sync::<$t>()
        );
    };
}

fn main() {
    print_impl_send_sync!(i32);
    print_impl_send_sync!(&i32);
    print_impl_send_sync!(&mut i32);
    print_impl_send_sync!(Box<i32>);
    print_impl_send_sync!(*mut i32);
    print_impl_send_sync!(NonNull<i32>);
    print_impl_send_sync!(Mutex<NonNull<i32>>);
    print_impl_send_sync!(String);
}
