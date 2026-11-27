use std::time::Duration;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct TargetGroup {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    // not in the spec's `required` list, so it can be missing or null
    pub group_id: Option<Uuid>,
}

/// GET `path` from the warpgate admin api and deserialize the body
async fn get_json<T: DeserializeOwned>(
    config: &Config,
    client: &reqwest::Client,
    path: &str,
) -> color_eyre::Result<T> {
    let url = format!(
        "{}/@warpgate/admin/api{}",
        config.wg_url.trim_end_matches('/'),
        path
    );
    tracing::debug!(%url, "requesting warpgate api");
    let response = client
        .get(&url)
        .header("X-Warpgate-Token", &config.token)
        .send()
        .await?;
    tracing::trace!(status = %response.status(), "warpgate responded");
    let body = response.error_for_status()?.text().await?;
    tracing::trace!(bytes = body.len(), "received response body");
    Ok(serde_json::from_str(&body)?)
}

/// send request to warpgate to list all the targets
pub async fn get_targets(
    config: &Config,
    client: &reqwest::Client,
) -> color_eyre::Result<Vec<Target>> {
    get_json(config, client, "/targets").await
}

/// send request to warpgate to list all the target groups
pub async fn get_target_groups(
    config: &Config,
    client: &reqwest::Client,
) -> color_eyre::Result<Vec<TargetGroup>> {
    get_json(config, client, "/target-groups").await
}

/// group name for targets that warpgate returns without a group_id
const UNGROUPED: &str = "ungrouped";

/// label each target as "group name - target name", sorted by group name.
/// targets without a group are placed under a runtime "ungrouped" group
pub fn map_groups(
    groups: Vec<TargetGroup>,
    targets: Vec<Target>,
) -> color_eyre::Result<Vec<(String, Target)>> {
    for group in &groups {
        tracing::trace!(id = %group.id, name = %group.name, "target group");
    }
    for target in &targets {
        tracing::trace!(
            id = %target.id,
            name = %target.name,
            group_id = ?target.group_id,
            "target"
        );
    }

    let mut grouped: Vec<(String, Target)> = Vec::new();

    for target in targets {
        let group_name = match target.group_id {
            None => {
                tracing::debug!(target = %target.name, "target has no group, using ungrouped");
                UNGROUPED.to_string()
            }
            Some(group_id) => {
                let Some(group) = groups.iter().find(|g| g.id == group_id) else {
                    color_eyre::eyre::bail!(
                        "target '{}' references group {} which was not returned by /target-groups",
                        target.name,
                        group_id
                    );
                };
                group.name.clone()
            }
        };
        grouped.push((group_name, target));
    }

    grouped.sort_by_key(|(group_name, target)| {
        (
            group_name.trim().to_lowercase(),
            target.name.trim().to_lowercase(),
        )
    });

    Ok(grouped
        .into_iter()
        .map(|(group_name, target)| (format!("{} - {}", group_name, target.name), target))
        .collect())
}

pub async fn get_nested_targets(config: &Config) -> color_eyre::Result<Vec<(String, Target)>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    tracing::debug!("fetching target groups and targets");
    let (groups, targets) = tokio::join!(
        get_target_groups(config, &client),
        get_targets(config, &client)
    );
    map_groups(groups?, targets?)
}
