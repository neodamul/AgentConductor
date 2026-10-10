use super::*;

pub(super) struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);
impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(super) async fn control_io(
    mut output: tokio::process::ChildStdout,
    input: tokio::process::ChildStdin,
    mut file: tokio::fs::File,
    initial: Vec<u8>,
    (mut handler, evaluation_limit): (AsyncControlHandler, Duration),
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    exited: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut input = Some(input);
    input
        .as_mut()
        .unwrap()
        .write_all(&initial)
        .await
        .map_err(|e| e.to_string())?;
    input
        .as_mut()
        .unwrap()
        .flush()
        .await
        .map_err(|e| e.to_string())?;
    let mut pending = Vec::new();
    let mut bytes = [0; 8192];
    loop {
        let n = output.read(&mut bytes).await.map_err(|e| e.to_string())?;
        if n == 0 {
            if !pending.is_empty() || input.is_some() {
                return Err(
                    "control stream ended before an explicit input close or with incomplete JSONL"
                        .into(),
                );
            }
            break;
        }
        // Evidence is persisted before evaluating any control message.
        file.write_all(&bytes[..n])
            .await
            .map_err(|e| e.to_string())?;
        file.flush().await.map_err(|e| e.to_string())?;
        pending.extend_from_slice(&bytes[..n]);
        while let Some(end) = pending.iter().position(|b| *b == b'\n') {
            if end > 1024 * 1024 {
                return Err("control JSONL message exceeds 1 MiB".into());
            }
            let message: serde_json::Value = serde_json::from_slice(&pending[..end])
                .map_err(|e| format!("invalid control JSONL: {e}"))?;
            pending.drain(..=end);
            let permission = message["type"] == "control_request";
            let evaluation =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(message)))
                    .map_err(|_| "control evaluator panicked")?;
            let mut evaluation = AbortOnDrop(tokio::spawn(evaluation));
            let cancel = async {
                while !cancelled.load(std::sync::atomic::Ordering::Acquire)
                    && !(permission && exited.load(std::sync::atomic::Ordering::Acquire))
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            };
            let reply = tokio::select! {
                biased;
                _=cancel=>return Err("control evaluation cancelled with the owned process".into()),
                result=&mut evaluation.0=>result.map_err(|_| "control evaluator panicked")??,
                _=tokio::time::sleep(evaluation_limit)=>return Err(format!("control evaluation exceeded {} seconds", evaluation_limit.as_secs())),
            };
            if cancelled.load(std::sync::atomic::Ordering::Acquire)
                || (permission && exited.load(std::sync::atomic::Ordering::Acquire))
            {
                return Err(
                    "control reply refused after process termination or cancellation".into(),
                );
            }
            if reply.messages.len() > 8 {
                return Err("control response exceeds eight messages".into());
            }
            let mut response = vec![];
            for value in reply.messages {
                if !value.is_object() {
                    return Err("control response must be a JSON object".into());
                }
                serde_json::to_writer(&mut response, &value).map_err(|e| e.to_string())?;
                response.push(b'\n');
                if response.len() > 256 * 1024 {
                    return Err("control response exceeds 256 KiB".into());
                }
            }
            if !response.is_empty() {
                let stdin = input
                    .as_mut()
                    .ok_or("control reply attempted after input was closed")?;
                stdin
                    .write_all(&response)
                    .await
                    .map_err(|e| e.to_string())?;
                stdin.flush().await.map_err(|e| e.to_string())?;
            }
            if reply.close_input
                && let Some(mut stdin) = input.take()
            {
                stdin.shutdown().await.map_err(|e| e.to_string())?;
            }
        }
        if pending.len() > 1024 * 1024 {
            return Err("control JSONL message exceeds 1 MiB".into());
        }
    }
    file.flush().await.map_err(|e| e.to_string())
}
