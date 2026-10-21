//! Retire a model's template in chap-core when it leaves the deployment:
//! `chap-devops models disable --retire`, and `chap-devops models remove`
//! unless `--keep-in-chap`.
//!
//! The template is found before anything changes, so a chap-core that does
//! not answer refuses the command while the model is still as it was. The
//! retire itself comes after the container is gone; when it fails, the
//! model is out of the deployment and the warning names the call to make.

use crate::api::Api;
use crate::configs::Template;
use crate::error::Result;
use crate::output::Report;
use crate::project::Project;
use serde::Serialize;

/// A retire that is ready: chap-core answered, and this is what it has.
pub(crate) struct Pending {
    api: Api,
    template: Option<Template>,
}

/// What the retire did, for `--json`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub(crate) enum Outcome {
    /// chap-core archived every version of this template.
    Retired { template_id: i64 },
    /// chap-core lists no live template of the model.
    NoTemplate,
    /// The request failed; the sentence says why.
    Failed { template_id: i64, error: String },
}

/// Look up the template of `service_id` before the model changes.
///
/// `None` when the deployment has no chap-core, which `required` turns into
/// the error that names the component. `way_out` is the flag that skips
/// the retire, for the refusal when chap-core does not answer.
pub(crate) fn prepare(
    project: &Project,
    id: &str,
    service_id: &str,
    required: bool,
    way_out: &str,
) -> Result<Option<Pending>> {
    if !project.state.components.has_chap_core_api() {
        if required {
            crate::components::require_chap_core(
                &project.state.components,
                "`chap-devops models disable --retire`",
            )?;
        }
        return Ok(None);
    }
    let api = crate::commands::configs::api_of(project);
    let template = crate::configs::template_named(&api, service_id).map_err(|err| {
        anyhow::anyhow!(
            "could not find the template of {id} in chap-core, so nothing changed: {err}; start \
             chap-core with `chap-devops up`, or {way_out}"
        )
    })?;
    Ok(Some(Pending { api, template }))
}

/// Retire the template that [`prepare`] found.
pub(crate) fn finish(pending: Option<Pending>) -> Option<Outcome> {
    let pending = pending?;
    let Some(template) = &pending.template else {
        return Some(Outcome::NoTemplate);
    };
    let retired = match crate::configs::retire(&pending.api, template.id) {
        Ok(()) => Outcome::Retired {
            template_id: template.id,
        },
        Err(err) => Outcome::Failed {
            template_id: template.id,
            error: err.to_string(),
        },
    };
    Some(retired)
}

/// The closing line of a retire.
pub(crate) fn line(id: &str, retired: &Outcome, lines: &mut Report) {
    match retired {
        Outcome::Retired { .. } => {
            lines.info(format!(
                "retired {id} in chap-core; its configured models are archived"
            ));
        }
        Outcome::NoTemplate => {
            lines.info(format!(
                "chap-core has no live template of {id}, so there was nothing to retire"
            ));
        }
        Outcome::Failed { template_id, error } => {
            lines.warning(format!(
                "could not retire {id} in chap-core: {error}; run `chap-devops api DELETE '{}'`",
                crate::configs::retire_path(*template_id)
            ));
        }
    }
}
