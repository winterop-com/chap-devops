//! What a registry refresh changed, against the copy in the cache.
//!
//! `chap-devops update` outside a deployment and `chap-devops registry update`
//! only fetch the registry, so the change is the whole answer: a count alone
//! reads the same whether the marketplace moved or not.

use super::Registry;
use serde::Serialize;

/// The models a refresh added, removed or changed.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Changes {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<Changed>,
}

/// One model whose entry changed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Changed {
    pub id: String,
    /// The stable version before and after; the same when only other
    /// fields of the entry changed.
    pub from: String,
    pub to: String,
}

impl Changes {
    /// Whether the refresh changed nothing.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// The changes as one clause: `new: a; changed: b 0.1.3 to 0.1.4`.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("new: {}", self.added.join(", ")));
        }
        if !self.changed.is_empty() {
            let changed: Vec<String> = self
                .changed
                .iter()
                .map(|c| match c.from == c.to {
                    true => c.id.clone(),
                    false => format!("{} {} to {}", c.id, c.from, c.to),
                })
                .collect();
            parts.push(format!("changed: {}", changed.join(", ")));
        }
        if !self.removed.is_empty() {
            parts.push(format!("removed: {}", self.removed.join(", ")));
        }
        parts.join("; ")
    }
}

/// What changed from `old` to `new`, in the order of the new index.
pub fn between(old: &Registry, new: &Registry) -> Changes {
    let mut changes = Changes::default();
    for model in &new.models {
        match old.models.iter().find(|m| m.id == model.id) {
            None => changes.added.push(model.id.clone()),
            Some(before) if as_json(before) != as_json(model) => changes.changed.push(Changed {
                id: model.id.clone(),
                from: before.channels.stable.clone(),
                to: model.channels.stable.clone(),
            }),
            Some(_) => {}
        }
    }
    for model in &old.models {
        if !new.models.iter().any(|m| m.id == model.id) {
            changes.removed.push(model.id.clone());
        }
    }
    changes
}

/// The closing line of a refresh: what changed, or that nothing did.
/// `models` is the count as words, such as `6 models`.
pub fn lines(
    models: &str,
    changes: Option<&Changes>,
    compared_age: Option<std::time::Duration>,
    lines: &mut crate::output::Report,
) {
    match changes {
        None => lines.info(format!("fetched the marketplace registry: {models}")),
        Some(changes) if changes.is_empty() => {
            lines.info(format!("the marketplace registry did not change: {models}"))
        }
        Some(changes) => lines.info(format!(
            "updated the marketplace registry: {models}; {}",
            changes.summary()
        )),
    };
    if let Some(age) = compared_age {
        lines.hint(format!(
            "compared with the cached copy from {} ago",
            crate::output::human_age(age)
        ));
    }
}

fn as_json(model: &super::Model) -> serde_json::Value {
    serde_json::to_value(model).unwrap_or_default()
}

#[cfg(test)]
mod tests;
