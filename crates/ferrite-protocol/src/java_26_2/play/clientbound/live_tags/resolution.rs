/// Resolves adapter-local member raw IDs against one configured registry snapshot.
///
/// Invalid IDs are omitted, while valid encounter order and duplicates are retained.
#[must_use]
pub fn resolve_members(raw_members: &[i32], registry_size: usize) -> Vec<usize> {
    raw_members
        .iter()
        .filter_map(|raw_id| usize::try_from(*raw_id).ok())
        .filter(|raw_id| *raw_id < registry_size)
        .collect()
}
