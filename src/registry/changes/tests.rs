use super::*;

fn registry() -> Registry {
    crate::registry::load_embedded().expect("embedded snapshot parses")
}

#[test]
fn the_same_registry_changes_nothing() {
    let changes = between(&registry(), &registry());
    assert!(changes.is_empty());
    assert_eq!(changes.summary(), "");
}

#[test]
fn a_new_stable_version_is_a_change_with_both_versions() {
    let old = registry();
    let mut new = registry();
    let id = new.models[0].id.clone();
    let from = new.models[0].channels.stable.clone();
    new.models[0].channels.stable = "9.9.9".to_string();
    let changes = between(&old, &new);
    assert_eq!(
        changes.changed,
        vec![Changed {
            id: id.clone(),
            from: from.clone(),
            to: "9.9.9".to_string(),
        }]
    );
    assert_eq!(changes.summary(), format!("changed: {id} {from} to 9.9.9"));
}

#[test]
fn another_field_is_a_change_without_versions() {
    let old = registry();
    let mut new = registry();
    new.models[1].summary.push_str(" More words.");
    let changes = between(&old, &new);
    assert_eq!(changes.summary(), format!("changed: {}", new.models[1].id));
}

#[test]
fn added_and_removed_models_are_named() {
    let old = registry();
    let mut new = registry();
    let gone = new.models.remove(0).id;
    let mut extra = new.models[0].clone();
    extra.id = "brand_new_model".to_string();
    new.models.push(extra);
    let changes = between(&old, &new);
    assert_eq!(changes.added, vec!["brand_new_model".to_string()]);
    assert_eq!(changes.removed, vec![gone.clone()]);
    assert_eq!(
        changes.summary(),
        format!("new: brand_new_model; removed: {gone}")
    );
}

#[test]
fn a_model_left_out_of_the_new_registry_is_not_removed() {
    let old = registry();
    let mut new = registry();
    let gone = new.models.remove(0).id;
    new.skipped.push(crate::registry::Skipped {
        path: format!("models/{gone}.yaml"),
        line: String::new(),
    });
    assert!(between(&old, &new).is_empty());
}
