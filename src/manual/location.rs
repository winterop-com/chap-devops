//! Where an image's registry answers, and how to get a pull token from it.
//!
//! ghcr hands an anonymous token out at a fixed path, which is what
//! [`super::ghcr::Client::anonymous`] asks. Every other registry says where
//! its token comes from in the `WWW-Authenticate` challenge of a 401 on
//! `/v2/`: Docker Hub, quay.io, GitLab and a registry of your own alike.

use super::Endpoints;
use super::ghcr::Client;
use super::source::GHCR_HOST;
use crate::error::Result;

/// The host Docker Hub's API answers on; `docker.io` is only its name.
pub const DOCKER_HUB_API: &str = "registry-1.docker.io";

/// The registry of one image, as its API addresses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The host the image is named after, such as `quay.io`.
    pub host: String,
    /// Where the API answers, such as `https://quay.io`.
    pub base: String,
    /// The repository inside it, such as `org/model`.
    pub repository: String,
}

impl Location {
    /// The registry of `image`, a reference without its tag or digest.
    pub fn of(image: &str, endpoints: &Endpoints) -> Location {
        let (host, path) = image.split_once('/').unwrap_or((GHCR_HOST, image));
        let host = host.to_lowercase();
        let docker_hub = matches!(
            host.as_str(),
            "docker.io" | "index.docker.io" | DOCKER_HUB_API
        );
        let repository = match docker_hub && !path.contains('/') {
            // Docker Hub keeps its official images under `library/`.
            true => format!("library/{path}"),
            false => path.to_string(),
        }
        .to_lowercase();
        let base = match (host.as_str(), &endpoints.oci_url) {
            (GHCR_HOST, _) => endpoints.ghcr_url.clone(),
            (_, Some(url)) => url.clone(),
            _ if docker_hub => format!("https://{DOCKER_HUB_API}"),
            _ => format!("https://{host}"),
        };
        Location {
            host,
            base,
            repository,
        }
    }

    /// A reader of this repository, with the pull token the registry hands
    /// out to anyone, or none when it asks for none.
    pub fn client(&self, endpoints: &Endpoints) -> Result<Client> {
        if self.host == GHCR_HOST {
            return Client::anonymous(&self.base, &self.repository, endpoints.timeout);
        }
        let url = format!("{}/v2/", self.base);
        let (status, challenge) = super::challenge(&url, endpoints.timeout)?;
        if (200..300).contains(&status) {
            return Ok(Client::open(
                &self.base,
                &self.repository,
                endpoints.timeout,
            ));
        }
        let Some(bearer) = challenge.as_deref().and_then(parse_bearer) else {
            return Err(anyhow::anyhow!(
                "{} wants credentials for every pull, and chap-devops asks only for public \
                 images; run `docker login {}` and `docker pull` the image, so its config \
                 comes from the local image",
                self.host,
                self.host
            ));
        };
        let token_url = format!(
            "{}{}service={}&scope=repository:{}:pull",
            bearer.realm,
            if bearer.realm.contains('?') { "&" } else { "?" },
            bearer.service,
            self.repository
        );
        let (status, body) = super::get(
            &token_url,
            endpoints.timeout,
            &[("Accept", "application/json")],
        )?;
        if !(200..300).contains(&status) {
            return Err(crate::error::ChapError::Http {
                url: token_url,
                status,
            }
            .into());
        }
        let token = super::ghcr::parse_token(&body)
            .map_err(|e| anyhow::anyhow!("reading {token_url}: {e}"))?;
        Ok(Client::with_token(
            &self.base,
            &self.repository,
            &token,
            endpoints.timeout,
        ))
    }
}

/// What the image `image` (without its tag) says about itself at `pin`, a
/// tag or a digest, or `None` when its registry has no such image.
pub fn image_config(
    image: &str,
    pin: &str,
    endpoints: &Endpoints,
) -> Result<Option<super::ghcr::ImageConfig>> {
    Location::of(image, endpoints)
        .client(endpoints)?
        .image_config(pin.trim_start_matches('@'))
}

/// The two fields of a `Bearer` challenge a token request needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bearer {
    pub realm: String,
    pub service: String,
}

/// `Bearer realm="https://auth.docker.io/token",service="registry.docker.io"`,
/// or `None` for a challenge of another kind, such as `Basic`.
pub fn parse_bearer(header: &str) -> Option<Bearer> {
    let (scheme, params) = header.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let mut realm = None;
    let mut service = String::new();
    for part in split_params(params) {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim().to_lowercase().as_str() {
            "realm" => realm = Some(value),
            "service" => service = value,
            _ => {}
        }
    }
    Some(Bearer {
        realm: realm.filter(|r| !r.is_empty())?,
        service,
    })
}

/// The comma-separated parameters of a challenge, where a quoted value may
/// hold a comma itself, as a `scope` with two actions does.
fn split_params(params: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut start, mut quoted) = (0, false);
    for (i, c) in params.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => {
                parts.push(&params[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&params[start..]);
    parts
}

#[cfg(test)]
mod tests;
