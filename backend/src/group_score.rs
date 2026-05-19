use std::collections::HashMap;

use crate::vertex::Vertex;

fn pair_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

pub fn group_scores(
    groups: &[Vec<usize>],
    vertices: &[Vertex],
    cache: &HashMap<(usize, usize), f64>,
) -> Vec<Option<f64>> {
    groups
        .iter()
        .map(|g| score_one_group(g, vertices, cache))
        .collect()
}

fn score_one_group(
    g: &[usize],
    vertices: &[Vertex],
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
            let s = if vertices[i].digest == vertices[j].digest {
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
    use std::path::PathBuf;

    fn make_vertex(id: u8) -> Vertex {
        Vertex {
            path: PathBuf::from(format!("/tmp/{id}")),
            digest: vec![id],
            features: None,
        }
    }

    #[test]
    fn hash_equal_only_group() {
        let v0 = make_vertex(1);
        let v1 = make_vertex(1);
        let groups = vec![vec![0, 1]];
        let vertices = vec![v0, v1];
        let cache = HashMap::new();
        let sc = group_scores(&groups, &vertices, &cache);
        assert_eq!(sc[0], Some(1.0));
    }
}
