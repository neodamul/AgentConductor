use agentconductor::gates::Receipt;

pub fn receipt_summary(result: Result<Option<Receipt>, String>) -> String {
    match result {
        Ok(Some(receipt)) => format!(
            "receipt={{id:{},status:{},task:{},created_at:{},expires_at:{:?}}}",
            receipt.id,
            receipt.status,
            receipt.context.task,
            receipt.created_at,
            receipt.expires_at
        ),
        Ok(None) => "receipt=None".into(),
        Err(error) => format!("receipt read failed: {error}"),
    }
}
