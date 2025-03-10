use std::sync::Arc;

pub trait AsyncIterator: Sync + Send {
    type Item: Send;

    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send;
}

struct Iter {
    items: Arc<Vec<i32>>,
    index: usize,
}

impl Iter {
    pub fn new(items: Arc<Vec<i32>>) -> Self {
        Self { items, index: 0 }
    }
}

impl AsyncIterator for Iter {
    type Item = i32;

    fn next(&mut self) -> impl Future<Output = Option<Self::Item>> + Send {
        async {
            while self.index < self.items.len() {
                let item = self.items[self.index];
                self.index += 1;
                return Some(item);
            }
            None
        }
    }
}

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread().build()?;

    rt.spawn(async {
        let items = Arc::new(vec![1, 2, 3, 4, 5]);
        let mut iter = Iter::new(items);

        while let Some(item) = iter.next().await {
            println!("{}", item);
        }
    });

    Ok(())
}
