use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use zeroclaw_config::schema::{Config, ModelRouteConfig, ResolvedPublicPeer};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublicPeerRouteSelection {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublicPeerExecutionContext {
    pub peer_id: String,
    pub default_route: Option<PublicPeerRouteSelection>,
    pub identity_overlay: Option<String>,
}

pub fn resolve_public_peer_runtime_route(
    model_routes: &[ModelRouteConfig],
    runtime_ref: &str,
) -> Option<PublicPeerRouteSelection> {
    let runtime_ref = runtime_ref.trim();
    if runtime_ref.is_empty() {
        return None;
    }

    let hint_ref = runtime_ref.strip_prefix("hint:").unwrap_or(runtime_ref);
    model_routes
        .iter()
        .find(|route| {
            route.hint.eq_ignore_ascii_case(hint_ref)
                || route.model.eq_ignore_ascii_case(runtime_ref)
        })
        .map(|route| PublicPeerRouteSelection {
            provider: route.provider.clone(),
            model: route.model.clone(),
            api_key: route.api_key.clone(),
        })
}

pub fn load_public_peer_identity_overlay(
    workspace_dir: &Path,
    peer_id: &str,
    identity_ref: &str,
) -> Result<Option<String>> {
    let identity_ref = identity_ref.trim();
    if identity_ref.is_empty() {
        return Ok(None);
    }

    let full_path = if Path::new(identity_ref).is_absolute() {
        PathBuf::from(identity_ref)
    } else {
        workspace_dir.join(identity_ref)
    };

    let mut rendered_overlay = None;
    let looks_like_json = full_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
    if looks_like_json {
        let identity_config = zeroclaw_config::schema::IdentityConfig {
            format: "aieos".into(),
            aieos_path: Some(identity_ref.to_string()),
            aieos_inline: None,
        };
        if let Ok(Some(aieos_identity)) =
            crate::identity::load_aieos_identity(&identity_config, workspace_dir)
        {
            let rendered = crate::identity::aieos_to_system_prompt(&aieos_identity);
            if !rendered.trim().is_empty() {
                rendered_overlay = Some(rendered);
            }
        }
    }

    let rendered_overlay = match rendered_overlay {
        Some(rendered) => rendered,
        None => {
            let contents = std::fs::read_to_string(&full_path).with_context(|| {
                format!(
                    "Failed to read peer identity overlay from {}",
                    full_path.display()
                )
            })?;
            let trimmed = contents.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            crate::util::truncate_with_ellipsis(
                trimmed,
                crate::agent::system_prompt::BOOTSTRAP_MAX_CHARS,
            )
        }
    };

    Ok(Some(format!(
        "## Peer Identity Overlay\n\nThis conversation is bound to public peer `{peer_id}`. Apply the following peer-specific identity instructions on top of the default workspace context.\n\n{rendered_overlay}"
    )))
}

pub fn resolve_public_peer_execution_context(
    config: &Config,
    resolved_public_peer: &ResolvedPublicPeer<'_>,
) -> PublicPeerExecutionContext {
    let mut execution = PublicPeerExecutionContext {
        peer_id: resolved_public_peer.peer_id.to_string(),
        ..PublicPeerExecutionContext::default()
    };

    let Some(peer) = resolved_public_peer.peer else {
        return execution;
    };

    if let Some(runtime_ref) = peer.runtime_ref.as_deref() {
        execution.default_route =
            resolve_public_peer_runtime_route(&config.providers.model_routes, runtime_ref);
        if execution.default_route.is_none() {
            tracing::warn!(
                public_peer = resolved_public_peer.peer_id,
                runtime_ref,
                "Ignoring unresolved public peer runtime_ref"
            );
        }
    }

    if let Some(identity_ref) = peer.identity_ref.as_deref() {
        match load_public_peer_identity_overlay(
            &config.workspace_dir,
            resolved_public_peer.peer_id,
            identity_ref,
        ) {
            Ok(overlay) => execution.identity_overlay = overlay,
            Err(err) => {
                tracing::warn!(
                    public_peer = resolved_public_peer.peer_id,
                    identity_ref,
                    "Failed to load public peer identity overlay: {err}"
                );
            }
        }
    }

    execution
}

pub fn normalize_target_public_peer(
    config: &Config,
    target_public_peer: Option<&str>,
) -> Result<Option<String>> {
    match target_public_peer {
        Some(peer_id) => Ok(Some(
            config
                .resolve_explicit_public_peer(peer_id)?
                .peer_id
                .to_string(),
        )),
        None => Ok(None),
    }
}

pub fn resolve_target_public_peer_execution_context(
    config: &Config,
    target_public_peer: Option<&str>,
) -> Result<Option<PublicPeerExecutionContext>> {
    match target_public_peer {
        Some(peer_id) => {
            let resolved_public_peer = config.resolve_explicit_public_peer(peer_id)?;
            Ok(Some(resolve_public_peer_execution_context(
                config,
                &resolved_public_peer,
            )))
        }
        None => Ok(None),
    }
}
