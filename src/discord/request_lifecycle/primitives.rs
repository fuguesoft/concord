use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(super) struct LastSelection<K> {
    selected: Option<K>,
}

impl<K> Default for LastSelection<K> {
    fn default() -> Self {
        Self { selected: None }
    }
}

impl<K> LastSelection<K>
where
    K: Copy + Eq,
{
    pub(super) fn clear(&mut self) {
        self.selected = None;
    }

    pub(super) fn select(&mut self, key: K) -> bool {
        let changed = self.selected != Some(key);
        self.selected = Some(key);
        changed
    }
}

#[derive(Debug)]
pub(super) struct TimedRequestSet<K> {
    requested: HashMap<K, Instant>,
    requested_order: VecDeque<K>,
    ttl: Duration,
    max_requested: usize,
}

impl<K> TimedRequestSet<K>
where
    K: Clone + Eq + Hash,
{
    pub(super) fn new(ttl: Duration, max_requested: usize) -> Self {
        Self {
            requested: HashMap::new(),
            requested_order: VecDeque::new(),
            ttl,
            max_requested,
        }
    }

    pub(super) fn insert(&mut self, key: K, now: Instant) -> bool {
        if self.requested.contains_key(&key) {
            return false;
        }
        self.requested.insert(key.clone(), now);
        self.requested_order.push_back(key);
        self.prune(now);
        true
    }

    pub(super) fn contains(&self, key: &K) -> bool {
        self.requested.contains_key(key)
    }

    pub(super) fn remove(&mut self, key: &K) {
        self.requested.remove(key);
        self.requested_order
            .retain(|requested_key| requested_key != key);
    }

    pub(super) fn prune(&mut self, now: Instant) {
        self.requested.retain(|_, requested_at| {
            now.checked_duration_since(*requested_at)
                .is_none_or(|age| age <= self.ttl)
        });
        self.requested_order
            .retain(|key| self.requested.contains_key(key));
        while self.requested.len() > self.max_requested {
            let Some(oldest) = self.requested_order.pop_front() else {
                break;
            };
            self.requested.remove(&oldest);
        }
    }
}
