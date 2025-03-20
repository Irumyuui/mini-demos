#![allow(unused)]

use std::sync::Arc;

use async_channel::{Receiver, Sender};

struct Bus {
    msg_ques: Arc<Vec<(Sender<String>, Receiver<String>)>>,
}

impl Bus {
    pub async fn new(len: usize) -> Self {
        let mut vec = Vec::with_capacity(len);
        while vec.len() < len {
            vec.push(async_channel::unbounded());
            let r = vec.last().unwrap().1.clone();
            let id = vec.len() - 1;
            // like a subscriber
            tokio::spawn(async move {
                while let Ok(msg) = r.recv().await {
                    println!("[{}] Received: {}", id, msg);
                }
            });
        }

        Self {
            msg_ques: Arc::new(vec),
        }
    }

    pub fn publish(&self, ids: impl Iterator<Item = usize>) -> Publisher {
        let mut vec = ids
            .map(|id| self.msg_ques.get(id))
            .filter(|opt| opt.is_some())
            .flatten()
            .map(|(s, _)| s.clone())
            .collect::<Vec<_>>();

        Publisher {
            senders: Arc::new(vec),
        }
    }
}

struct Publisher {
    senders: Arc<Vec<Sender<String>>>,
}

impl Publisher {
    pub async fn send(&self, msg: String) {
        for s in self.senders.iter() {
            s.send(msg.clone()).await.unwrap();
        }
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        futures::executor::block_on(async {
            self.send("Closed".to_string()).await;
        });
    }
}

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread().build()?;

    rt.block_on(async {
        let bus = Bus::new(6).await;

        let p = bus.publish(0..3);
        p.send("Hello, World!".to_string()).await;
    });

    Ok(())
}
