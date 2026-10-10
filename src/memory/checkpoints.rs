use crate::{
    cooperative::{Acknowledgment, Address, Directive, Progress},
    workflow,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "protocol", deny_unknown_fields)]
pub enum Envelope {
    #[serde(rename = "agentconductor.checkpoint.v1")]
    Directive { directive: Directive },
    #[serde(rename = "agentconductor.ack.v1")]
    Acknowledgment { acknowledgment: Acknowledgment },
    #[serde(rename = "agentconductor.progress.v1")]
    Progress { progress: Progress },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub memory_id: String,
    pub project: String,
    pub agent_id: String,
    pub content_hash: String,
}
impl Envelope {
    pub fn address(&self) -> &Address {
        match self {
            Self::Directive { directive } => &directive.address,
            Self::Acknowledgment { acknowledgment } => &acknowledgment.address,
            Self::Progress { progress } => &progress.address,
        }
    }
    pub(crate) fn scope(&self) -> Result<(String, String), String> {
        address_scope(self.address())
    }
}

/// Run project and address-digest agent tag used for publication and discovery.
pub fn address_scope(a: &Address) -> Result<(String, String), String> {
    if a.run_id.len() != 32
        || !a.run_id.bytes().all(|c| c.is_ascii_hexdigit())
        || a.plan_digest.len() != 64
        || !a.plan_digest.bytes().all(|c| c.is_ascii_hexdigit())
        || a.attempt == 0
        || [&a.agent, &a.task, &a.session_id]
            .iter()
            .any(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
    {
        return Err("invalid checkpoint transport address".into());
    }
    Ok((
        format!("agentconductor.run.{}", a.run_id),
        format!(
            "agentconductor.{}",
            workflow::digest(
                &serde_json::to_vec(a).map_err(|_| "checkpoint address serialization failed")?
            )
        ),
    ))
}

/// Import only observations from an exact memory ID. A remote directive can never create local authority.
pub fn import(
    store: &mut crate::state::Store,
    client: &super::Client,
    memory_id: &str,
    now: u64,
) -> Result<serde_json::Value, String> {
    let packet = client.read(memory_id)?;
    if matches!(packet, Envelope::Directive { .. }) {
        return Err(
            "memory directives must reconcile with local authority; import is not allowed".into(),
        );
    }
    let key = format!("memory_checkpoint_import/{memory_id}");
    let fingerprint = serde_json::json!({"memory_id":memory_id,"content_hash":workflow::digest(&serde_json::to_vec(&packet).map_err(|_|"checkpoint serialization failed")?)});
    if let Some(prior) = store.optional_document(&key)? {
        if prior != fingerprint {
            return Err("imported memory content changed".into());
        }
        return Ok(serde_json::json!({"memory_id":memory_id,"applied":false,"duplicate":true}));
    }
    let applied = match packet {
        Envelope::Acknowledgment { acknowledgment } => {
            let before =
                store.optional_document(&format!("cooperative_directive/{}", acknowledgment.id))?;
            let after = crate::cooperative::acknowledge(store, acknowledgment, now)?;
            before.as_ref()
                != Some(
                    &serde_json::to_value(after).map_err(|_| "checkpoint serialization failed")?,
                )
        }
        Envelope::Progress { progress } => crate::cooperative::report_progress_from(
            store,
            progress,
            now,
            "agentmemory exact-ID adapter",
        )?,
        Envelope::Directive { .. } => unreachable!(),
    };
    store.record_once(&key,&fingerprint,"Imported an exact agentmemory checkpoint observation; execution and approval permissions unchanged")?;
    Ok(serde_json::json!({"memory_id":memory_id,"applied":applied,"duplicate":false}))
}

/// DD-44: import new observations addressed to `address` without operator-supplied IDs.
/// Every candidate still goes through the exact-ID `import`; directives are never imported.
pub fn discover(
    store: &mut crate::state::Store,
    client: &super::Client,
    address: &Address,
    now: u64,
) -> Result<serde_json::Value, String> {
    let (project, agent_id) = address_scope(address)?;
    let (mut imported, mut already, mut directives, mut rejected) = (0, 0, 0, vec![]);
    for id in client.list_scope(&project, &agent_id)? {
        if store
            .optional_document(&format!("memory_checkpoint_import/{id}"))?
            .is_some()
        {
            already += 1;
            continue;
        }
        match import(store, client, &id, now) {
            Ok(result) => imported += usize::from(result["applied"] == true),
            Err(e) if e.starts_with("memory directives") => directives += 1,
            Err(error) => rejected.push(serde_json::json!({"memory_id":id,"error":error})),
        }
    }
    Ok(
        serde_json::json!({"imported":imported,"already_imported":already,"skipped_directives":directives,"rejected":rejected}),
    )
}
