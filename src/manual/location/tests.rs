use super::*;

fn endpoints() -> Endpoints {
    Endpoints::default()
}

#[test]
fn ghcr_goes_to_the_ghcr_endpoint() {
    let at = Location::of("ghcr.io/Chap-Models/model", &endpoints());
    assert_eq!(at.host, "ghcr.io");
    assert_eq!(at.base, "https://ghcr.io");
    assert_eq!(at.repository, "chap-models/model");
}

#[test]
fn docker_hub_answers_on_its_api_host_and_keeps_official_images_under_library() {
    let at = Location::of("docker.io/python", &endpoints());
    assert_eq!(at.base, "https://registry-1.docker.io");
    assert_eq!(at.repository, "library/python");
    let at = Location::of("docker.io/someone/model", &endpoints());
    assert_eq!(at.repository, "someone/model");
}

#[test]
fn another_registry_answers_on_its_own_host_or_on_the_test_hook() {
    let at = Location::of("quay.io/org/model", &endpoints());
    assert_eq!(at.base, "https://quay.io");
    assert_eq!(at.repository, "org/model");
    let at = Location::of("registry.example.org:5000/team/model", &endpoints());
    assert_eq!(at.base, "https://registry.example.org:5000");

    let hooked = Endpoints {
        oci_url: Some("http://127.0.0.1:18001".to_string()),
        ..Endpoints::default()
    };
    assert_eq!(
        Location::of("quay.io/org/model", &hooked).base,
        "http://127.0.0.1:18001"
    );
    // ghcr keeps its own hook.
    assert_eq!(
        Location::of("ghcr.io/org/model", &hooked).base,
        "https://ghcr.io"
    );
}

#[test]
fn a_bearer_challenge_names_the_realm_and_the_service() {
    let bearer = parse_bearer(
        r#"Bearer realm="https://auth.docker.io/token",service="registry.docker.io",scope="repository:library/python:pull,push""#,
    )
    .expect("a bearer challenge");
    assert_eq!(bearer.realm, "https://auth.docker.io/token");
    assert_eq!(bearer.service, "registry.docker.io");
}

#[test]
fn a_challenge_of_another_kind_is_none() {
    assert_eq!(parse_bearer(r#"Basic realm="registry""#), None);
    assert_eq!(parse_bearer("Bearer service=\"x\""), None);
    assert_eq!(parse_bearer(""), None);
}
