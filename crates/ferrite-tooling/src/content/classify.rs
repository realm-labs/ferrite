use crate::content::model::{Category, Family};
use anyhow::{Context as _, Result, ensure};
use ferrite_registry::bundle::CatalogClassification;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeSet;

pub(crate) struct Classifier<'a> {
    category: &'a Category,
    selectors: Vec<Selector>,
}

struct Selector {
    exact: BTreeSet<String>,
    patterns: GlobSet,
}

impl<'a> Classifier<'a> {
    pub(crate) fn compile(
        category: &'a Category,
        ids: &BTreeSet<String>,
        blocks: &BTreeSet<String>,
    ) -> Result<Self> {
        let mut selectors = Vec::with_capacity(category.family.len());
        for family in &category.family {
            validate_family_policy(category, family, ids, blocks)?;
            let exact = family.exact.iter().map(|value| normalize(value)).collect();
            let mut patterns = GlobSetBuilder::new();
            for pattern in &family.patterns {
                patterns.add(Glob::new(&normalize(pattern)).with_context(|| {
                    format!(
                        "compile {}/{} pattern {pattern}",
                        category.kind, family.name
                    )
                })?);
            }
            selectors.push(Selector {
                exact,
                patterns: patterns.build()?,
            });
        }
        Ok(Self {
            category,
            selectors,
        })
    }

    pub(crate) fn classify(&self, id: &str, blocks: &BTreeSet<String>) -> Result<&'a Family> {
        let mut matches = Vec::new();
        for (family, selector) in self.category.family.iter().zip(&self.selectors) {
            let mut matched = selector.exact.contains(id) || selector.patterns.is_match(id);
            if !matched && family.block_items && matches.is_empty() {
                matched = blocks.contains(id);
            }
            if !matched && family.remaining {
                matched = matches.is_empty();
            }
            if matched {
                matches.push(family);
            }
        }
        ensure!(
            matches.len() == 1,
            "{} {id} matched {} catalog families",
            self.category.kind,
            matches.len()
        );
        Ok(matches[0])
    }
}

fn validate_family_policy(
    category: &Category,
    family: &Family,
    ids: &BTreeSet<String>,
    blocks: &BTreeSet<String>,
) -> Result<()> {
    ensure!(
        !(family.remaining && family.classification == CatalogClassification::Special),
        "{}/{} is a Special fallback",
        category.kind,
        family.name
    );
    if family.remaining && family.classification == CatalogClassification::DataOnly {
        ensure!(
            matches!(
                category.kind.as_str(),
                "potion" | "recipe" | "loot_table" | "advancement" | "damage_type" | "enchantment"
            ),
            "{}/{} is not an approved data-only fallback",
            category.kind,
            family.name
        );
    }
    for exact in &family.exact {
        ensure!(
            ids.contains(&normalize(exact)),
            "{}/{} has stale exact identity {exact}",
            category.kind,
            family.name
        );
    }
    for pattern in &family.patterns {
        let pattern = Glob::new(&normalize(pattern))?.compile_matcher();
        ensure!(
            ids.iter().any(|id| pattern.is_match(id)),
            "{}/{} pattern matches no identities",
            category.kind,
            family.name
        );
    }
    if family.block_items {
        ensure!(
            category.kind == "item" && ids.iter().any(|id| blocks.contains(id)),
            "{}/{} block-item selector matches no identities",
            category.kind,
            family.name
        );
    }
    ensure!(
        family.remaining
            || family.block_items
            || !family.exact.is_empty()
            || !family.patterns.is_empty(),
        "{}/{} has no selector",
        category.kind,
        family.name
    );
    Ok(())
}

fn normalize(value: &str) -> String {
    if value.contains(':') {
        value.to_owned()
    } else {
        format!("minecraft:{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family(name: &str, exact: &[&str], patterns: &[&str], remaining: bool) -> Family {
        Family {
            name: name.to_owned(),
            classification: CatalogClassification::BehaviorFamily,
            rules: vec!["BLK-001".to_owned()],
            exact: exact.iter().map(|value| (*value).to_owned()).collect(),
            patterns: patterns.iter().map(|value| (*value).to_owned()).collect(),
            block_items: false,
            remaining,
        }
    }

    #[test]
    fn explicit_and_pattern_selectors_precede_remaining() {
        let category = Category {
            kind: "block".to_owned(),
            expected_count: 3,
            ids_sha1: "unused".to_owned(),
            family: vec![
                family("exact", &["stone"], &[], false),
                family("stairs", &[], &["*_stairs"], false),
                family("remaining", &[], &[], true),
            ],
        };
        let ids = ["minecraft:stone", "minecraft:oak_stairs", "minecraft:dirt"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let classifier = Classifier::compile(&category, &ids, &BTreeSet::new()).unwrap();
        assert_eq!(
            classifier
                .classify("minecraft:stone", &BTreeSet::new())
                .unwrap()
                .name,
            "exact"
        );
        assert_eq!(
            classifier
                .classify("minecraft:oak_stairs", &BTreeSet::new())
                .unwrap()
                .name,
            "stairs"
        );
        assert_eq!(
            classifier
                .classify("minecraft:dirt", &BTreeSet::new())
                .unwrap()
                .name,
            "remaining"
        );
    }
}
