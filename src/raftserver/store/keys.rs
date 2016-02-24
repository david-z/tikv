use std::vec::Vec;

use byteorder::{ByteOrder, BigEndian, WriteBytesExt};

use raftserver::{Result, other};
use std::mem;

pub const MIN_KEY: &'static [u8] = &[];
pub const MAX_KEY: &'static [u8] = &[0xFF];

// local is in (0x01, 0x02);
pub const LOCAL_PREFIX: u8 = 0x01;
pub const LOCAL_MIN_KEY: &'static [u8] = &[LOCAL_PREFIX];
pub const LOCAL_MAX_KEY: &'static [u8] = &[LOCAL_PREFIX + 1];
pub const META1_PREFIX: u8 = 0x02;
pub const META2_PREFIX: u8 = 0x03;
pub const META1_PREFIX_KEY: &'static [u8] = &[META1_PREFIX];
pub const META2_PREFIX_KEY: &'static [u8] = &[META2_PREFIX];
pub const META_MIN_KEY: &'static [u8] = &[META1_PREFIX];
pub const META_MAX_KEY: &'static [u8] = &[META2_PREFIX + 1];
pub const META1_MAX_KEY: &'static [u8] = &[META1_PREFIX, MAX_KEY[0]];
pub const META2_MAX_KEY: &'static [u8] = &[META2_PREFIX, MAX_KEY[0]];

pub const DATA_PREFIX: u8 = b'z';
pub const DATA_PREFIX_KEY: &'static [u8] = &[DATA_PREFIX];

// Following keys are all local keys, so the first byte must be 0x01.
pub const STORE_IDENT_KEY: &'static [u8] = &[LOCAL_PREFIX, 0x01];
// We save two types region data in DB, for raft and other meta data.
// When the store starts, we should iterate all region meta data to
// construct peer, no need to travel large raft data, so we separate them
// with different prefixes.
pub const REGION_RAFT_PREFIX: u8 = 0x02;
pub const REGION_RAFT_PREFIX_KEY: &'static [u8] = &[LOCAL_PREFIX, REGION_RAFT_PREFIX];
pub const REGION_META_PREFIX: u8 = 0x03;
pub const REGION_META_PREFIX_KEY: &'static [u8] = &[LOCAL_PREFIX, REGION_META_PREFIX];
pub const REGION_META_MIN_KEY: &'static [u8] = &[LOCAL_PREFIX, REGION_META_PREFIX];
pub const REGION_META_MAX_KEY: &'static [u8] = &[LOCAL_PREFIX, REGION_META_PREFIX + 1];

// Following are the suffix after the local prefix.
// For region id
pub const RAFT_LOG_SUFFIX: u8 = 0x01;
pub const RAFT_HARD_STATE_SUFFIX: u8 = 0x02;
pub const RAFT_APPLIED_INDEX_SUFFIX: u8 = 0x03;
pub const RAFT_LAST_INDEX_SUFFIX: u8 = 0x04;
pub const RAFT_TRUNCATED_STATE_SUFFIX: u8 = 0x05;

// For region meta
pub const REGION_INFO_SUFFIX: u8 = 0x01;
pub const REGION_TOMBSTONE_SUFFIX: u8 = 0x02;

pub fn store_ident_key() -> Vec<u8> {
    STORE_IDENT_KEY.to_vec()
}

fn make_region_id_key(region_id: u64, suffix: u8, extra_cap: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(REGION_RAFT_PREFIX_KEY.len() + mem::size_of::<u64>() +
                                     mem::size_of::<u8>() +
                                     extra_cap);
    key.extend_from_slice(REGION_RAFT_PREFIX_KEY);
    // no need check error here, can't panic;
    key.write_u64::<BigEndian>(region_id).unwrap();
    key.push(suffix);
    key
}

pub fn region_raft_prefix(region_id: u64) -> Vec<u8> {
    let mut key = Vec::with_capacity(REGION_RAFT_PREFIX_KEY.len() + mem::size_of::<u64>());
    key.extend_from_slice(REGION_RAFT_PREFIX_KEY);
    // no need check error here, can't panic;
    key.write_u64::<BigEndian>(region_id).unwrap();
    key
}

pub fn raft_log_key(region_id: u64, log_index: u64) -> Vec<u8> {
    let mut key = make_region_id_key(region_id, RAFT_LOG_SUFFIX, mem::size_of::<u64>());
    // no need check error here, can't panic;
    key.write_u64::<BigEndian>(log_index).unwrap();
    key
}

pub fn raft_log_prefix(region_id: u64) -> Vec<u8> {
    make_region_id_key(region_id, RAFT_LOG_SUFFIX, 0)
}

pub fn raft_hard_state_key(region_id: u64) -> Vec<u8> {
    make_region_id_key(region_id, RAFT_HARD_STATE_SUFFIX, 0)
}

pub fn raft_applied_index_key(region_id: u64) -> Vec<u8> {
    make_region_id_key(region_id, RAFT_APPLIED_INDEX_SUFFIX, 0)
}

pub fn raft_last_index_key(region_id: u64) -> Vec<u8> {
    make_region_id_key(region_id, RAFT_LAST_INDEX_SUFFIX, 0)
}

pub fn raft_truncated_state_key(region_id: u64) -> Vec<u8> {
    make_region_id_key(region_id, RAFT_TRUNCATED_STATE_SUFFIX, 0)
}

fn make_region_meta_key(region_id: u64, suffix: u8) -> Vec<u8> {
    let mut key = Vec::with_capacity(REGION_META_PREFIX_KEY.len() + mem::size_of::<u64>() +
                                     mem::size_of::<u8>());
    key.extend_from_slice(REGION_META_PREFIX_KEY);
    // no need to check error here, can't panic;
    key.write_u64::<BigEndian>(region_id).unwrap();
    key.push(suffix);
    key
}

// Decode region meta key, return the region key and meta suffix type.
pub fn decode_region_meta_key(key: &[u8]) -> Result<(u64, u8)> {
    if REGION_META_PREFIX_KEY.len() + mem::size_of::<u64>() + mem::size_of::<u8>() != key.len() {
        return Err(other(format!("invalid region meta key length for key {:?}", key)));
    }

    if !key.starts_with(REGION_META_PREFIX_KEY) {
        return Err(other(format!("invalid region meta prefix for key {:?}", key)));
    }

    let region_id =
        BigEndian::read_u64(&key[REGION_META_PREFIX_KEY.len()..REGION_META_PREFIX_KEY.len() +
                                                               mem::size_of::<u64>()]);

    Ok((region_id, key[key.len() - 1]))
}

pub fn region_meta_prefix(region_id: u64) -> Vec<u8> {
    let mut key = Vec::with_capacity(REGION_META_PREFIX_KEY.len() + mem::size_of::<u64>());
    key.extend_from_slice(REGION_META_PREFIX_KEY);
    key.write_u64::<BigEndian>(region_id).unwrap();
    key
}

pub fn region_info_key(region_id: u64) -> Vec<u8> {
    make_region_meta_key(region_id, REGION_INFO_SUFFIX)
}

// When a peer is destroyed, we would record current region max peer id as
// the tombstone value, any peer for this region with a peer id <= tombstone
// value is not allowed to create in this store.
pub fn region_tombstone_key(region_id: u64) -> Vec<u8> {
    make_region_meta_key(region_id, REGION_TOMBSTONE_SUFFIX)
}

// Returns a region route meta (meta1, meta2) indexing key for the
// given key.
// For data key, it returns a meta2 key, e.g, "zabc" -> \0x03"zabc"
// For meta2 key, it returns a meta1 key, e.g, \0x03\"zabc" -> \0x02"zabc"
// For meta1 key, it returns a MIN_KEY, e.g, \x02\"zabc" -> ""
pub fn region_route_meta_key(key: &[u8]) -> Vec<u8> {
    if key.len() == 0 {
        return MIN_KEY.to_vec();
    }

    match key[0] {
        META1_PREFIX => MIN_KEY.to_vec(),
        META2_PREFIX => vec![META1_PREFIX_KEY, &key[1..]].concat(),
        _ => vec![META2_PREFIX_KEY, key].concat(),
    }
}

pub fn validate_region_route_meta_key(key: &[u8]) -> Result<()> {
    if key == MIN_KEY {
        return Ok(());
    }

    // TODO: Maybe no necessary to check this, remove later?
    if key.len() < META1_PREFIX_KEY.len() {
        return Err(other(format!("{:?} is too short", key)));
    }

    let prefix = key[0];
    if prefix != META2_PREFIX && prefix != META1_PREFIX {
        return Err(other(format!("{:?} is not a meta key", key)));
    }

    // TODO: check data prefix later?
    if MAX_KEY < &key[META1_PREFIX_KEY.len()..] {
        return Err(other(format!("{:?} is > {:?}", key, MAX_KEY)));
    }

    Ok(())
}

pub fn validate_data_key(key: &[u8]) -> Result<()> {
    if !key.starts_with(DATA_PREFIX_KEY) {
        return Err(other(format!("invalid data key {:?}, must start with {}",
                                 key,
                                 DATA_PREFIX)));
    }

    Ok(())
}

pub fn data_key(key: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(DATA_PREFIX_KEY.len() + key.len());
    v.extend_from_slice(DATA_PREFIX_KEY);
    v.extend_from_slice(key);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_region_id_key() {
        let region_ids = vec![0, 1, 1024, ::std::u64::MAX];
        for region_id in region_ids {
            let prefix = region_raft_prefix(region_id);

            assert!(raft_log_prefix(region_id).starts_with(&prefix));
            assert!(raft_log_key(region_id, 1).starts_with(&prefix));
            assert!(raft_hard_state_key(region_id).starts_with(&prefix));
            assert!(raft_applied_index_key(region_id).starts_with(&prefix));
            assert!(raft_last_index_key(region_id).starts_with(&prefix));
            assert!(raft_truncated_state_key(region_id).starts_with(&prefix));
        }

        // test sort.
        let tbls = vec![(1, 0, Ordering::Greater), (1, 1, Ordering::Equal), (1, 2, Ordering::Less)];
        for (lid, rid, order) in tbls {
            let lhs = region_raft_prefix(lid);
            let rhs = region_raft_prefix(rid);
            assert_eq!(lhs.partial_cmp(&rhs), Some(order));
        }
    }

    #[test]
    fn test_raft_log_sort() {
        let tbls = vec![(1, 1, 1, 2, Ordering::Less),
                        (2, 1, 1, 2, Ordering::Greater),
                        (1, 1, 1, 1, Ordering::Equal)];

        for (lid, l_log_id, rid, r_log_id, order) in tbls {
            let lhs = raft_log_key(lid, l_log_id);
            let rhs = raft_log_key(rid, r_log_id);
            assert_eq!(lhs.partial_cmp(&rhs), Some(order));
        }
    }

    #[test]
    fn test_region_meta_key() {
        let ids: Vec<u64> = vec![1, 1024, u64::max_value()];
        for id in ids {
            let prefix = region_meta_prefix(id);
            let info_key = region_info_key(id);
            assert!(info_key.starts_with(&prefix));

            assert_eq!(decode_region_meta_key(&info_key).unwrap(),
                       (id, REGION_INFO_SUFFIX));
        }

        // test sort.
        let tbls: Vec<(u64, u64, Ordering)> = vec![
        (1, 2, Ordering::Less),
        (1, 1, Ordering::Equal),
        (2, 1, Ordering::Greater),
        ];

        for (lkey, rkey, order) in tbls {
            let lhs = region_info_key(lkey);
            let rhs = region_info_key(rkey);
            assert_eq!(lhs.partial_cmp(&rhs), Some(order));
        }
    }

    fn route_key(prefix: u8, key: &[u8]) -> Vec<u8> {
        let mut v = vec![];
        v.push(prefix);
        v.extend_from_slice(key);
        v
    }

    fn meta1_key(key: &[u8]) -> Vec<u8> {
        route_key(META1_PREFIX, key)
    }

    fn meta2_key(key: &[u8]) -> Vec<u8> {
        route_key(META2_PREFIX, key)
    }

    #[test]
    fn test_region_route_meta_key() {
        let dkey = data_key(b"abc");
        let tbls = vec![
            (vec![], vec![]),
            (meta1_key(&dkey), vec![]),
            (meta2_key(&dkey), meta1_key(&dkey)),
            (dkey.clone(), meta2_key(&dkey)),
        ];

        for (lkey, rkey) in tbls {
            assert_eq!(region_route_meta_key(&lkey), rkey);
        }

        let tbls = vec![
            (vec![], true),
            (vec![0xFF], false),
            (vec![DATA_PREFIX, 1], false),
            (vec![META1_PREFIX - 1, 1], false),
            (vec![META2_PREFIX + 1, 1], false),
            (vec![META1_PREFIX, 1], true),
            (vec![META2_PREFIX, 1], true),
            (vec![META1_PREFIX, 0xFF, 1], false),
            (vec![META2_PREFIX, 0xFF, 1], false),
        ];

        for (key, ok) in tbls {
            assert_eq!(validate_region_route_meta_key(&key).is_ok(), ok);
        }
    }

    #[test]
    fn test_data_key() {
        validate_data_key(&data_key(b"abc")).unwrap();
        validate_data_key(b"abc").unwrap_err();
    }
}