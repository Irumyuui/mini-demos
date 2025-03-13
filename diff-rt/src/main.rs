use std::time::Duration;

struct Foo {
    sender: async_channel::Sender<String>,
}

impl Foo {
    async fn send(&self, msg: String) {
        self.sender.send(msg).await.unwrap();
    }

    async fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded::<String>();
        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if msg.is_empty() {
                    break;
                }
                println!("Received: {}", msg);
            }
        });
        Foo { sender }
    }

    async fn drop_async(&self) {
        self.send("".into()).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Dropped!");
    }
}

impl Drop for Foo {
    fn drop(&mut self) {
        futures::executor::block_on(async {
            self.drop_async().await;
        })
    }
}

#[tokio::main]
async fn main() {
    futures::executor::block_on(async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Ok!")
    });

    let foo = Foo::new().await;
    foo.send("Hello".into()).await;
    drop(foo);
}
