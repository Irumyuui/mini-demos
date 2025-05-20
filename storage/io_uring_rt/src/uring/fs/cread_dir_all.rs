use std::{path::Path, pin::Pin};

use rustix::fs::Mode;

use crate::uring::{fs::File, op::Op};

#[derive(Debug)]
struct Inner {
    mode: rustix::fs::Mode,
}

impl Inner {
    fn new() -> Self {
        Self {
            mode: Mode::from(0o777),
        }
    }

    async fn mkdir<P: AsRef<Path>>(&self, p: P) -> std::io::Result<()> {
        Op::mkdir(p, self.mode)?.complete().await
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

async fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    let Ok(res) = File::open(path).await else {
        return false;
    };

    let metadata = res.metadata().await;
    match metadata {
        Ok(m) => m.is_dir(),
        Err(_) => false,
    }
}

pub struct DirBuilder {
    inner: Inner,
    recursive: bool,
}

impl DirBuilder {
    pub fn new() -> Self {
        Self {
            inner: Inner::new(),
            recursive: false,
        }
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    pub fn mode(&mut self, mode: Mode) -> &mut Self {
        self.inner.set_mode(mode);
        self
    }

    pub async fn create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        if self.recursive {
            self.recurse_create_dir_all(path.as_ref()).await
        } else {
            self.inner.mkdir(path).await
        }
    }

    fn recurse_create_dir_all<'a>(
        &'a self,
        path: &'a Path,
    ) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + 'a>> {
        Box::pin(async move {
            if path == Path::new("") {
                return Ok(());
            }

            match self.inner.mkdir(path).await {
                Ok(_) => return Ok(()),
                Err(_) if is_dir(path).await => return Ok(()),
                Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            match path.parent() {
                Some(p) => self.recurse_create_dir_all(p).await?,
                None => {
                    return Err(std::io::Error::other("create dir all failed"));
                }
            }

            match self.inner.mkdir(path).await {
                Ok(()) => Ok(()),
                Err(_) if is_dir(path).await => Ok(()),
                Err(e) => Err(e),
            }
        })
    }
}
