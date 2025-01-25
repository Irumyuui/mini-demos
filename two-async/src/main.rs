use tokio::task::block_in_place;

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()?;

    let (tx, rx) = tokio::sync::mpsc::channel::<usize>(2);

    runtime.spawn(async move {
        let mut rx = rx;
        while let Some(msg) = rx.recv().await {
            if msg == 0 {
                println!("exit");
                return;
            }
            println!("msg: {}", msg);
        }
    });

    let rt2 = tokio::runtime::Builder::new_current_thread().build()?;

    rt2.block_on(async move {
        tx.send(1).await.unwrap();
        tx.send(0).await.unwrap();
    });

    Ok(())
}
