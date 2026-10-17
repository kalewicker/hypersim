//! Safe generational arena with the legacy SlotMap wire layout and an opt-in
//! two-lane allocator. Private: graph keys remain the existing SlotMap key types.
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use slotmap::{Key, KeyData};
use std::marker::PhantomData;

/// Explicit continuation metadata, separate from the legacy snapshot wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct AllocationState {
    free: [Vec<u32>; 2],
    next: [u64; 2],
    isolated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Slot<V> {
    // Field order is part of the existing binary SlotMap wire representation.
    value: Option<V>,
    version: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Arena<K, V> {
    slots: Vec<Slot<V>>,
    free: [Vec<u32>; 2],
    next: [u64; 2],
    len: usize,
    isolated: bool,
    key: PhantomData<K>,
}

impl<K: Key, V> Arena<K, V> {
    pub fn new(isolated: bool) -> Self {
        Self {
            // Marker 1/None is deliberately invalid for legacy SlotMap readers.
            slots: vec![Slot {
                value: None,
                version: u32::from(isolated),
            }],
            free: Default::default(),
            next: [1, 2],
            len: 0,
            isolated,
            key: PhantomData,
        }
    }

    pub fn isolated(&self) -> bool {
        self.isolated
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn allocation_state(&self) -> AllocationState {
        AllocationState { free: self.free.clone(), next: self.next, isolated: self.isolated }
    }

    pub(crate) fn restore_allocation_state(&mut self, state: AllocationState) -> Result<(), &'static str> {
        if state.isolated != self.isolated || state.next != self.next {
            return Err("allocator mode/frontier mismatch");
        }
        for lane in 0..2 {
            let mut expected = self.free[lane].clone();
            let mut supplied = state.free[lane].clone();
            expected.sort_unstable(); supplied.sort_unstable();
            if expected != supplied { return Err("allocator vacant-slot inventory mismatch"); }
        }
        self.free = state.free;
        Ok(())
    }

    fn lane(&self, index: u32) -> usize {
        if self.isolated {
            usize::from(index % 2 == 0)
        } else {
            0
        }
    }

    fn key(index: u32, version: u32) -> K {
        KeyData::from_ffi(((version as u64) << 32) | index as u64).into()
    }

    fn index(key: K) -> u32 {
        key.data().as_ffi() as u32
    }
    fn version(key: K) -> u32 {
        (key.data().as_ffi() >> 32) as u32
    }

    pub fn insert(&mut self, value: V, metadata: bool) -> K {
        let lane = usize::from(self.isolated && metadata);
        let index = if let Some(index) = self.free[lane].pop() {
            index
        } else {
            let index = self.next[lane];
            assert!(index < u32::MAX as u64, "generational arena is full");
            if index as usize >= self.slots.len() {
                self.slots.resize_with(index as usize + 1, || Slot {
                    value: None,
                    version: 0,
                });
            }
            self.next[lane] += if self.isolated { 2 } else { 1 };
            index as u32
        };
        let slot = &mut self.slots[index as usize];
        debug_assert!(slot.value.is_none());
        slot.version |= 1;
        slot.value = Some(value);
        self.len += 1;
        Self::key(index, slot.version)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        if Self::index(key) == 0 {
            return None;
        }
        let slot = self.slots.get(Self::index(key) as usize)?;
        if slot.version != Self::version(key) {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        if Self::index(key) == 0 {
            return None;
        }
        let slot = self.slots.get_mut(Self::index(key) as usize)?;
        if slot.version != Self::version(key) {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let index = Self::index(key);
        let lane = self.lane(index);
        if index == 0 {
            return None;
        }
        let slot = self.slots.get_mut(index as usize)?;
        if slot.version != Self::version(key) || slot.value.is_none() {
            return None;
        }
        assert!(
            !self.isolated || slot.version != u32::MAX,
            "isolated arena generation exhausted"
        );
        let value = slot.value.take();
        slot.version = slot.version.wrapping_add(1);
        self.free[lane].push(index);
        self.len -= 1;
        value
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.slots
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(index, slot)| {
                slot.value
                    .as_ref()
                    .map(|v| (Self::key(index as u32, slot.version), v))
            })
    }
}

impl<K, V: Serialize> Serialize for Arena<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.slots.serialize(serializer)
    }
}

impl<'de, K: Key, V: Deserialize<'de>> Deserialize<'de> for Arena<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let slots = Vec::<Slot<V>>::deserialize(deserializer)?;
        if slots.is_empty() || slots.len() >= u32::MAX as usize {
            return Err(D::Error::custom("invalid arena size"));
        }
        if slots[0].value.is_some() || slots[0].version > 1 {
            return Err(D::Error::custom("unknown arena marker"));
        }
        let isolated = slots[0].version == 1;
        let mut result = Self::new(isolated);
        // SlotMap restores its vacant slots in ascending order onto a LIFO list.
        // In isolated mode zero-version lane padding is never an allocated slot.
        let mut gap = [false; 2];
        for (index, slot) in slots.iter().enumerate().skip(1) {
            if (slot.version % 2 == 1) != slot.value.is_some() {
                return Err(D::Error::custom("inconsistent arena occupation"));
            }
            let lane = result.lane(index as u32);
            if isolated && slot.version == 0 {
                gap[lane] = true;
                continue;
            }
            if isolated && gap[lane] {
                return Err(D::Error::custom("noncontiguous allocated lane"));
            }
            result.next[lane] = index as u64 + if isolated { 2 } else { 1 };
            if slot.value.is_some() {
                result.len += 1;
            } else {
                result.free[lane].push(index as u32);
            }
        }
        result.slots = slots;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use slotmap::SlotMap;

    #[test]
    fn legacy_matches_real_slotmap_keys_bytes_reuse_and_restore() {
        for seed in 0..8 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut a = Arena::<NodeId, u64>::new(false);
            let mut reference = SlotMap::<NodeId, u64>::with_key();
            let mut keys = Vec::new();
            for value in 0..1000 {
                if keys.is_empty() || rng.gen_bool(0.55) {
                    let k = a.insert(value, rng.gen());
                    assert_eq!(k, reference.insert(value));
                    keys.push(k);
                } else {
                    let k = keys[rng.gen_range(0..keys.len())];
                    assert_eq!(a.remove(k), reference.remove(k));
                    assert_eq!(a.get(k), reference.get(k));
                }
                assert_eq!(
                    a.iter().collect::<Vec<_>>(),
                    reference.iter().collect::<Vec<_>>()
                );
                let bytes = bincode::serialize(&a).unwrap();
                assert_eq!(bytes, bincode::serialize(&reference).unwrap());
                if value % 100 == 0 {
                    a = bincode::deserialize(&bytes).unwrap();
                    reference = bincode::deserialize(&bytes).unwrap();
                }
            }
        }
    }

    #[test]
    fn unequal_metadata_histories_cannot_change_world_keys() {
        let mut a = Arena::<NodeId, u64>::new(true);
        let mut b = a.clone();
        let mut rng = ChaCha8Rng::seed_from_u64(91);
        let mut world = Vec::new();
        let mut meta = Vec::new();
        for value in 0..1000 {
            for _ in 0..rng.gen_range(0..8) {
                if !meta.is_empty() && rng.gen_bool(0.4) {
                    let k = meta.swap_remove(rng.gen_range(0..meta.len()));
                    b.remove(k).unwrap();
                } else {
                    meta.push(b.insert(value, true));
                }
            }
            let k = a.insert(value, false);
            assert_eq!(k, b.insert(value, false));
            world.push(k);
            if rng.gen_bool(0.5) {
                let k = world.swap_remove(rng.gen_range(0..world.len()));
                assert_eq!(a.remove(k), b.remove(k));
            }
            for &k in &world {
                assert_eq!(a.get(k), b.get(k));
            }
            if value % 100 == 0 {
                a = bincode::deserialize(&bincode::serialize(&a).unwrap()).unwrap();
                b = bincode::deserialize(&bincode::serialize(&b).unwrap()).unwrap();
            }
        }
        assert!(
            bincode::deserialize::<SlotMap<NodeId, u64>>(&bincode::serialize(&b).unwrap()).is_err()
        );
    }

    #[test]
    fn malformed_isolated_payloads_and_exhaustion_fail_closed() {
        for slots in [
            vec![Slot {
                value: None::<u64>,
                version: 2,
            }],
            vec![
                Slot {
                    value: None,
                    version: 1,
                },
                Slot {
                    value: Some(1),
                    version: 2,
                },
            ],
            vec![
                Slot {
                    value: None,
                    version: 1,
                },
                Slot {
                    value: None,
                    version: 0,
                },
                Slot {
                    value: None,
                    version: 0,
                },
                Slot {
                    value: Some(1),
                    version: 1,
                },
            ],
        ] {
            assert!(bincode::deserialize::<Arena<NodeId, u64>>(
                &bincode::serialize(&slots).unwrap()
            )
            .is_err());
        }
        let mut a = Arena::<NodeId, u64>::new(true);
        let k = a.insert(1, false);
        a.slots[1].version = u32::MAX;
        let exhausted = Arena::<NodeId, u64>::key(1, u32::MAX);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| a.remove(exhausted))).is_err()
        );
        assert_eq!(a.get(exhausted), Some(&1));
        assert!(a.get(k).is_none());
    }
}
