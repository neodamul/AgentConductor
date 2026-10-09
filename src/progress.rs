use crate::{
    protocol::{Decoder, Provider},
    state::Store,
};
use std::{future::Future, path::Path, time::Duration};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

struct Reader {
    offset: u64,
    decoder: Decoder,
    store: Store,
}

impl Reader {
    async fn poll(
        &mut self,
        path: &Path,
        task: &str,
        attempt: u64,
        finished: bool,
    ) -> Result<bool, String> {
        let mut file = match tokio::fs::File::open(path).await {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && !finished => return Ok(false),
            Err(e) => return Err(format!("read owned task spool: {e}")),
        };
        if file.metadata().await.map_err(|e| e.to_string())?.len() < self.offset {
            return Err("owned task spool was truncated".into());
        }
        file.seek(std::io::SeekFrom::Start(self.offset))
            .await
            .map_err(|e| e.to_string())?;
        let mut bytes = [0; 64 * 1024];
        let n = file.read(&mut bytes).await.map_err(|e| e.to_string())?;
        self.offset = self
            .offset
            .checked_add(n as u64)
            .ok_or("task spool offset overflow")?;
        let events = if n == 0 && finished {
            self.decoder.finish()?
        } else {
            self.decoder.push(&bytes[..n])?
        };
        self.store.append_task_events(task, attempt, &events)?;
        Ok(n > 0)
    }
}

/// Owns the worker future so observation failure/cancellation drops its process group.
pub async fn observe<F: Future>(
    future: F,
    spool: &Path,
    database: &Path,
    task: &str,
    attempt: u64,
    provider: Provider,
) -> Result<F::Output, String> {
    let mut reader = Reader {
        offset: 0,
        decoder: Decoder::new(provider, 1024 * 1024),
        store: Store::open(database)?,
    };
    tokio::pin!(future);
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            result=&mut future=>{
                // A launch failure can legitimately have no stdout spool.
                if spool.exists() {
                    while reader.poll(spool,task,attempt,true).await? {}
                }
                return Ok(result);
            },
            _=interval.tick()=>{reader.poll(spool,task,attempt,false).await?;},
        }
    }
}
