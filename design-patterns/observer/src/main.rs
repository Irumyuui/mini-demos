#![allow(unused)]

use std::rc::{Rc, Weak};

trait Observer {
    fn id(&self) -> usize;

    fn update(&self, message: String);
}

trait Suject {
    fn attach(&mut self, observer: &Rc<dyn Observer>);

    fn detach(&mut self, observer: &dyn Observer);

    fn notify(&self, message: String);
}

struct BasicSubject {
    observers: Vec<Weak<dyn Observer>>,
}

impl Default for BasicSubject {
    fn default() -> Self {
        Self {
            observers: Default::default(),
        }
    }
}

impl Suject for BasicSubject {
    fn attach(&mut self, observer: &Rc<dyn Observer>) {
        println!("Attaching observer: {}", observer.id());
        self.observers.push(Rc::downgrade(observer));
    }

    fn detach(&mut self, observer: &dyn Observer) {
        println!("Detaching observer: {}", observer.id());

        self.observers
            .retain(|o| o.upgrade().is_some_and(|o| o.id() != observer.id()));
    }

    fn notify(&self, message: String) {
        for o in self.observers.iter() {
            if let Some(o) = o.upgrade() {
                o.update(message.clone());
            }
        }
    }
}

struct BasicObserver {
    id: usize,
}

impl BasicObserver {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

impl Observer for BasicObserver {
    fn id(&self) -> usize {
        self.id
    }

    fn update(&self, message: String) {
        println!("Observer {} received message: {}", self.id, message);
    }
}

fn main() {
    let mut subject = BasicSubject::default();

    let observers: Vec<Rc<dyn Observer>> = (0..4)
        .map(|id| Rc::new(BasicObserver::new(id)) as Rc<dyn Observer>)
        .collect();
    for o in observers.iter() {
        subject.attach(o);
    }

    subject.notify("NOTIFY FIRST".to_string());
    subject.detach(observers[2].as_ref());
    subject.notify("NOTIFY SECOND".to_string());
}
