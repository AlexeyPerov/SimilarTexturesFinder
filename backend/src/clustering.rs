use std::collections::HashMap;
use std::path::PathBuf;

pub struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl Dsu {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) {
        let mut ra = self.find(a);
        let mut rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        if self.rank[ra] == self.rank[rb] {
            self.rank[ra] += 1;
        }
    }
}

/// Build connected components (1 group per tree). Groups sorted by lexicographic path order.
pub fn build_groups(
    n: usize,
    hash_edges: &[(usize, usize)],
    composite_edges: &[(usize, usize)],
    paths: &[PathBuf],
) -> Vec<Vec<usize>> {
    let mut dsu = Dsu::new(n);
    for &(i, j) in hash_edges {
        dsu.union(i, j);
    }
    for &(i, j) in composite_edges {
        dsu.union(i, j);
    }

    let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let r = dsu.find(i);
        buckets.entry(r).or_default().push(i);
    }

    let mut groups: Vec<Vec<usize>> = buckets.into_values().collect();
    for g in &mut groups {
        g.sort_by(|a, b| paths[*a].to_string_lossy().cmp(&paths[*b].to_string_lossy()));
    }
    groups.sort_by(|ga, gb| {
        paths[ga[0]]
            .to_string_lossy()
            .cmp(&paths[gb[0]].to_string_lossy())
    });
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unions_merge() {
        let mut d = Dsu::new(3);
        d.union(0, 1);
        assert_eq!(d.find(0), d.find(1));
        assert_ne!(d.find(0), d.find(2));
    }
}
