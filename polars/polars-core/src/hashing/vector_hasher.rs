use arrow::bitmap::utils::get_bit_unchecked;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use polars_arrow::utils::CustomIterTools;
use polars_utils::HashSingle;
use rayon::prelude::*;
use xxhash_rust::xxh3::xxh3_64_with_seed;

use super::*;
use crate::datatypes::UInt64Chunked;
use crate::prelude::*;
use crate::utils::arrow::array::Array;
use crate::POOL;

// Read more:
//  https://www.cockroachlabs.com/blog/vectorized-hash-joiner/
//  http://myeyesareblind.com/2017/02/06/Combine-hash-values/

pub trait VecHash {
    /// Compute the hash for all values in the array.
    ///
    /// This currently only works with the AHash RandomState hasher builder.
    fn vec_hash(&self, _random_state: RandomState, _buf: &mut Vec<u64>) {
        unimplemented!()
    }

    fn vec_hash_combine(&self, _random_state: RandomState, _hashes: &mut [u64]) {
        unimplemented!()
    }
}

pub(crate) fn get_null_hash_value(random_state: RandomState) -> u64 {
    // we just start with a large prime number and hash that twice
    // to get a constant hash value for null/None
    let mut hasher = random_state.build_hasher();
    3188347919usize.hash(&mut hasher);
    let first = hasher.finish();
    let mut hasher = random_state.build_hasher();
    first.hash(&mut hasher);
    hasher.finish()
}

fn insert_null_hash(chunks: &[ArrayRef], random_state: RandomState, buf: &mut Vec<u64>) {
    let null_h = get_null_hash_value(random_state);
    let hashes = buf.as_mut_slice();

    let mut offset = 0;
    chunks.iter().for_each(|arr| {
        if arr.null_count() > 0 {
            let validity = arr.validity().unwrap();
            let (slice, byte_offset, _) = validity.as_slice();
            (0..validity.len())
                .map(|i| unsafe { get_bit_unchecked(slice, i + byte_offset) })
                .zip(&mut hashes[offset..])
                .for_each(|(valid, h)| {
                    *h = [null_h, *h][valid as usize];
                })
        }
        offset += arr.len();
    });
}

fn integer_vec_hash<T>(ca: &ChunkedArray<T>, random_state: RandomState, buf: &mut Vec<u64>)
where
    T: PolarsIntegerType,
    T::Native: Hash + FxHash,
{
    // Note that we don't use the no null branch! This can break in unexpected ways.
    // for instance with threading we split an array in n_threads, this may lead to
    // splits that have no nulls and splits that have nulls. Then one array is hashed with
    // Option<T> and the other array with T.
    // Meaning that they cannot be compared. By always hashing on Option<T> the random_state is
    // the only deterministic seed.
    buf.clear();
    buf.reserve(ca.len());

    let k = random_state.hash_one(FXHASH_K);

    #[allow(unused_unsafe)]
    #[allow(clippy::useless_transmute)]
    ca.downcast_iter().for_each(|arr| {
        buf.extend(
            arr.values()
                .as_slice()
                .iter()
                .copied()
                .map(|v| unsafe { v._fx_hash(k) }),
        );
    });
    insert_null_hash(&ca.chunks, random_state, buf)
}

fn integer_vec_hash_combine<T>(ca: &ChunkedArray<T>, random_state: RandomState, hashes: &mut [u64])
where
    T: PolarsIntegerType,
    T::Native: Hash + FxHash,
{
    let null_h = get_null_hash_value(random_state.clone());
    let k = random_state.hash_one(FXHASH_K);

    let mut offset = 0;
    ca.downcast_iter().for_each(|arr| {
        match arr.null_count() {
            0 => arr
                .values()
                .as_slice()
                .iter()
                .zip(&mut hashes[offset..])
                .for_each(|(v, h)| {
                    let l = v._fx_hash(k);
                    *h = _boost_hash_combine(l, *h)
                }),
            _ => {
                let validity = arr.validity().unwrap();
                let (slice, byte_offset, _) = validity.as_slice();
                (0..validity.len())
                    .map(|i| unsafe { get_bit_unchecked(slice, i + byte_offset) })
                    .zip(&mut hashes[offset..])
                    .zip(arr.values().as_slice())
                    .for_each(|((valid, h), l)| {
                        let l = l._fx_hash(k);
                        *h = _boost_hash_combine([null_h, l][valid as usize], *h)
                    });
            }
        }
        offset += arr.len();
    });
}

macro_rules! vec_hash_int {
    ($ca:ident, $fx_hash:ident) => {
        impl VecHash for $ca {
            fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
                integer_vec_hash(self, random_state, buf)
            }

            fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
                integer_vec_hash_combine(self, random_state, hashes)
            }
        }
    };
}

vec_hash_int!(Int64Chunked, fx_hash_64_bit);
vec_hash_int!(Int32Chunked, fx_hash_32_bit);
vec_hash_int!(Int16Chunked, fx_hash_16_bit);
vec_hash_int!(Int8Chunked, fx_hash_8_bit);
vec_hash_int!(UInt64Chunked, fx_hash_64_bit);
vec_hash_int!(UInt32Chunked, fx_hash_32_bit);
vec_hash_int!(UInt16Chunked, fx_hash_16_bit);
vec_hash_int!(UInt8Chunked, fx_hash_8_bit);

impl VecHash for Utf8Chunked {
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        self.as_binary().vec_hash(random_state, buf)
    }

    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        self.as_binary().vec_hash_combine(random_state, hashes)
    }
}

impl VecHash for BinaryChunked {
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        buf.clear();
        buf.reserve(self.len());
        let null_h = get_null_hash_value(random_state);

        self.downcast_iter().for_each(|arr| {
            if arr.null_count() == 0 {
                // simply use the null_hash as seed to get a hash determined by `random_state` that is passed
                buf.extend(arr.values_iter().map(|v| xxh3_64_with_seed(v, null_h)))
            } else {
                buf.extend(arr.into_iter().map(|opt_v| match opt_v {
                    Some(v) => xxh3_64_with_seed(v, null_h),
                    None => null_h,
                }))
            }
        });
    }

    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        let null_h = get_null_hash_value(random_state);

        let mut offset = 0;
        self.downcast_iter().for_each(|arr| {
            match arr.null_count() {
                0 => arr
                    .values_iter()
                    .zip(&mut hashes[offset..])
                    .for_each(|(v, h)| {
                        let l = xxh3_64_with_seed(v, null_h);
                        *h = _boost_hash_combine(l, *h)
                    }),
                _ => {
                    let validity = arr.validity().unwrap();
                    let (slice, byte_offset, _) = validity.as_slice();
                    (0..validity.len())
                        .map(|i| unsafe { get_bit_unchecked(slice, i + byte_offset) })
                        .zip(&mut hashes[offset..])
                        .zip(arr.values_iter())
                        .for_each(|((valid, h), l)| {
                            let l = if valid {
                                xxh3_64_with_seed(l, null_h)
                            } else {
                                null_h
                            };
                            *h = _boost_hash_combine(l, *h)
                        });
                }
            }
            offset += arr.len();
        });
    }
}

impl VecHash for BooleanChunked {
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        buf.clear();
        buf.reserve(self.len());
        let true_h = random_state.hash_single(true);
        let false_h = random_state.hash_single(false);
        let null_h = get_null_hash_value(random_state);
        self.downcast_iter().for_each(|arr| {
            if arr.null_count() == 0 {
                buf.extend(arr.values_iter().map(|v| if v { true_h } else { false_h }))
            } else {
                buf.extend(arr.into_iter().map(|opt_v| match opt_v {
                    Some(true) => true_h,
                    Some(false) => false_h,
                    None => null_h,
                }))
            }
        });
    }

    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        let true_h = random_state.hash_single(true);
        let false_h = random_state.hash_single(false);
        let null_h = get_null_hash_value(random_state);

        let mut offset = 0;
        self.downcast_iter().for_each(|arr| {
            match arr.null_count() {
                0 => arr
                    .values_iter()
                    .zip(&mut hashes[offset..])
                    .for_each(|(v, h)| {
                        let l = if v { true_h } else { false_h };
                        *h = _boost_hash_combine(l, *h)
                    }),
                _ => {
                    let validity = arr.validity().unwrap();
                    let (slice, byte_offset, _) = validity.as_slice();
                    (0..validity.len())
                        .map(|i| unsafe { get_bit_unchecked(slice, i + byte_offset) })
                        .zip(&mut hashes[offset..])
                        .zip(arr.values())
                        .for_each(|((valid, h), l)| {
                            let l = if valid {
                                if l {
                                    true_h
                                } else {
                                    false_h
                                }
                            } else {
                                null_h
                            };
                            *h = _boost_hash_combine(l, *h)
                        });
                }
            }
            offset += arr.len();
        });
    }
}

impl VecHash for Float32Chunked {
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        self.bit_repr_small().vec_hash(random_state, buf)
    }

    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        self.bit_repr_small().vec_hash_combine(random_state, hashes)
    }
}
impl VecHash for Float64Chunked {
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        self.bit_repr_large().vec_hash(random_state, buf)
    }
    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        self.bit_repr_large().vec_hash_combine(random_state, hashes)
    }
}

#[cfg(feature = "object")]
impl<T> VecHash for ObjectChunked<T>
where
    T: PolarsObject,
{
    fn vec_hash(&self, random_state: RandomState, buf: &mut Vec<u64>) {
        // Note that we don't use the no null branch! This can break in unexpected ways.
        // for instance with threading we split an array in n_threads, this may lead to
        // splits that have no nulls and splits that have nulls. Then one array is hashed with
        // Option<T> and the other array with T.
        // Meaning that they cannot be compared. By always hashing on Option<T> the random_state is
        // the only deterministic seed.
        buf.clear();
        buf.reserve(self.len());

        self.downcast_iter().for_each(|arr| {
            buf.extend(arr.into_iter().map(|opt_v| {
                let mut hasher = random_state.build_hasher();
                opt_v.hash(&mut hasher);
                hasher.finish()
            }))
        });
    }

    fn vec_hash_combine(&self, random_state: RandomState, hashes: &mut [u64]) {
        self.apply_to_slice(
            |opt_v, h| {
                let mut hasher = random_state.build_hasher();
                opt_v.hash(&mut hasher);
                _boost_hash_combine(hasher.finish(), *h)
            },
            hashes,
        )
    }
}

/// Contains a ptr to the string slice an the precomputed hash of that string.
/// During rehashes, we will rehash the hash instead of the string, that makes rehashing
/// cheap and allows cache coherent small hash tables.
#[derive(Eq, Copy, Clone, Debug)]
pub(crate) struct BytesHash<'a> {
    payload: Option<&'a [u8]>,
    pub(super) hash: u64,
}

impl<'a> Hash for BytesHash<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}

impl<'a> BytesHash<'a> {
    #[inline]
    pub(crate) fn new(s: Option<&'a [u8]>, hash: u64) -> Self {
        Self { payload: s, hash }
    }
}

impl<'a> PartialEq for BytesHash<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.hash == other.hash) && (self.payload == other.payload)
    }
}

pub(crate) fn prepare_hashed_relation_threaded<T, I>(
    iters: Vec<I>,
) -> Vec<HashMap<T, (bool, Vec<IdxSize>), RandomState>>
where
    I: Iterator<Item = T> + Send + TrustedLen,
    T: Send + Hash + Eq + Sync + Copy,
{
    let n_partitions = iters.len();
    assert!(n_partitions.is_power_of_two());

    let (hashes_and_keys, build_hasher) = create_hash_and_keys_threaded_vectorized(iters, None);

    // We will create a hashtable in every thread.
    // We use the hash to partition the keys to the matching hashtable.
    // Every thread traverses all keys/hashes and ignores the ones that doesn't fall in that partition.
    POOL.install(|| {
        (0..n_partitions)
            .into_par_iter()
            .map(|partition_no| {
                let build_hasher = build_hasher.clone();
                let hashes_and_keys = &hashes_and_keys;
                let partition_no = partition_no as u64;
                let mut hash_tbl: HashMap<T, (bool, Vec<IdxSize>), RandomState> =
                    HashMap::with_hasher(build_hasher);

                let n_threads = n_partitions as u64;
                let mut offset = 0;
                for hashes_and_keys in hashes_and_keys {
                    let len = hashes_and_keys.len();
                    hashes_and_keys
                        .iter()
                        .enumerate()
                        .for_each(|(idx, (h, k))| {
                            let idx = idx as IdxSize;
                            // partition hashes by thread no.
                            // So only a part of the hashes go to this hashmap
                            if this_partition(*h, partition_no, n_threads) {
                                let idx = idx + offset;
                                let entry = hash_tbl
                                    .raw_entry_mut()
                                    // uses the key to check equality to find and entry
                                    .from_key_hashed_nocheck(*h, k);

                                match entry {
                                    RawEntryMut::Vacant(entry) => {
                                        entry.insert_hashed_nocheck(*h, *k, (false, vec![idx]));
                                    }
                                    RawEntryMut::Occupied(mut entry) => {
                                        let (_k, v) = entry.get_key_value_mut();
                                        v.1.push(idx);
                                    }
                                }
                            }
                        });

                    offset += len as IdxSize;
                }
                hash_tbl
            })
            .collect()
    })
}

pub(crate) fn create_hash_and_keys_threaded_vectorized<I, T>(
    iters: Vec<I>,
    build_hasher: Option<RandomState>,
) -> (Vec<Vec<(u64, T)>>, RandomState)
where
    I: IntoIterator<Item = T> + Send,
    I::IntoIter: TrustedLen,
    T: Send + Hash + Eq,
{
    let build_hasher = build_hasher.unwrap_or_default();
    let hashes = POOL.install(|| {
        iters
            .into_par_iter()
            .map(|iter| {
                // create hashes and keys
                iter.into_iter()
                    .map(|val| {
                        let mut hasher = build_hasher.build_hasher();
                        val.hash(&mut hasher);
                        (hasher.finish(), val)
                    })
                    .collect_trusted::<Vec<_>>()
            })
            .collect()
    });
    (hashes, build_hasher)
}

pub(crate) fn df_rows_to_hashes_threaded(
    keys: &[DataFrame],
    hasher_builder: Option<RandomState>,
) -> PolarsResult<(Vec<UInt64Chunked>, RandomState)> {
    let hasher_builder = hasher_builder.unwrap_or_default();

    let hashes = POOL.install(|| {
        keys.into_par_iter()
            .map(|df| {
                let hb = hasher_builder.clone();
                let (ca, _) = df_rows_to_hashes(df, Some(hb))?;
                Ok(ca)
            })
            .collect::<PolarsResult<Vec<_>>>()
    })?;
    Ok((hashes, hasher_builder))
}

pub(crate) fn df_rows_to_hashes(
    keys: &DataFrame,
    build_hasher: Option<RandomState>,
) -> PolarsResult<(UInt64Chunked, RandomState)> {
    let build_hasher = build_hasher.unwrap_or_default();

    let mut iter = keys.iter();
    let first = iter.next().expect("at least one key");
    let mut hashes = vec![];
    first.vec_hash(build_hasher.clone(), &mut hashes)?;
    let hslice = hashes.as_mut_slice();

    for keys in iter {
        keys.vec_hash_combine(build_hasher.clone(), hslice)?;
    }

    let chunks = vec![Box::new(PrimitiveArray::new(
        ArrowDataType::UInt64,
        hashes.into(),
        None,
    )) as ArrayRef];
    unsafe { Ok((UInt64Chunked::from_chunks("", chunks), build_hasher)) }
}
