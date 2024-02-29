use hashbrown::hash_map as base;
use core::hash::{BuildHasher, Hasher, SipHasher};
use core::hash::Hash;
use spinlock::SpinNoIrq;
use arceos_api::time;
use core::hash::SipHasher13;

static PARK_MILLER_LEHMER_SEED: SpinNoIrq<u32> = SpinNoIrq::new(0);
const RAND_MAX: u64 = 2_147_483_647;

pub struct MyHasher(SipHasher13);

impl Hasher for MyHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        // for byte in bytes {
        //     self.value = self.value.wrapping_add(*byte as u64);
        // }
        self.0.write(bytes)
    }
}

// pub struct MyHashBuilder;

// impl BuildHasher for MyHashBuilder {
//     type Hasher = MyHasher;

//     fn build_hasher(&self) -> Self::Hasher {
//         MyHasher { value: 0 }
//     }
// }

pub struct MyRandomState {
    k0: u64,
    k1: u64,
}

impl Default for MyRandomState {
    fn default() -> Self {
        let ran = random();
        MyRandomState {k0: ran as u64, k1: (ran >> 64) as u64}
    }
}

pub fn random() -> u128 {
    let mut seed = PARK_MILLER_LEHMER_SEED.lock();
    if *seed == 0 {
        *seed = time::ax_current_time().subsec_millis() as u32;
    }

    let mut ret: u128 = 0;
    for _ in 0..4 {
        *seed = ((u64::from(*seed) * 48271) % RAND_MAX) as u32;
        ret = (ret << 32) | (*seed as u128);
    }
    ret
}

pub struct HashMap<K, V, S = MyRandomState> {
    base: base::HashMap<K, V, S>,
}

pub struct MyIter<'a, K: 'a, V: 'a> {
    base: base::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for MyIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}


impl BuildHasher for MyRandomState {
    type Hasher = MyHasher;

    fn build_hasher(&self) -> Self::Hasher {
        MyHasher(SipHasher13::new_with_keys(0, 0))
    }
}

impl<K, V> HashMap<K, V, MyRandomState> {
    pub fn new() -> HashMap<K, V, MyRandomState> {
        Default::default()
    }
}

impl<K, V, S> HashMap<K, V, S> {
    pub const fn with_hasher(hash_builder: S) -> HashMap<K, V, S> {
        HashMap { base: base::HashMap::with_hasher(hash_builder) }
    }
    pub fn iter(&self) -> MyIter<'_, K, V> {
        MyIter { base: self.base.iter() }
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    S: Default,
{
    /// Creates an empty `HashMap<K, V, S>`, with the `Default` value for the hasher.
    #[inline]
    fn default() -> HashMap<K, V, S> {
        HashMap::with_hasher(Default::default())
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.base.insert(k, v)
    }
}

