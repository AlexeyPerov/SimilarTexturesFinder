use std::collections::HashMap;

fn pair_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

pub fn group_scores(
    groups: &[Vec<usize>],
    digests: &[Vec<u8>],
    cache: &HashMap<(usize, usize), f64>,
) -> Vec<Option<f64>> {
    groups
        .iter()
        .map(|g| score_one_group(g, digests, cache))
        .collect()
}

fn score_one_group(
    g: &[usize],
    digests: &[Vec<u8>],
    cache: &HashMap<(usize, usize), f64>,
) -> Option<f64> {
    if g.len() < 2 {
        return None;
    }

    let mut best: Option<f64> = None;
    for ai in 0..g.len() {
        for bi in (ai + 1)..g.len() {
            let i = g[ai];
            let j = g[bi];
            let s = if digests[i] == digests[j] {
                Some(1.0)
            } else {
                cache.get(&pair_key(i, j)).copied()
            };
            if let Some(v) = s {
                best = Some(best.map_or(v, |b| b.max(v)));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_equal_only_group() {
        let groups = vec![vec![0, 1]];
        let digests = vec![vec![1], vec![1]];
        let cache = HashMap::new();
        let sc = group_scores(&groups, &digests, &cache);
        assert_eq!(sc[0], Some(1.0));
    }
}
