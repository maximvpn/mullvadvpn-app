use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use bytes::{Buf, Bytes, BytesMut};

#[derive(Default)]
pub struct Fragments {
    fragment_map: BTreeMap<u16, Vec<Fragment>>,
}

#[derive(Debug)]
pub struct PayloadTooSmall;

impl Fragments {
    // TODO: Let caller provide output buffer.
    pub fn insert(&mut self, mut payload: Bytes) -> Result<Option<Bytes>, PayloadTooSmall> {
        let id = payload.try_get_u16().map_err(|_| PayloadTooSmall)?;
        let index = payload.try_get_u8().map_err(|_| PayloadTooSmall)?;
        let fragment_count = payload.try_get_u8().map_err(|_| PayloadTooSmall)?;
        let fragment = Fragment {
            index,
            payload,
            time_received: Instant::now(),
        };

        let fragments = self.fragment_map.entry(id).or_insert(vec![]);
        fragments.push(fragment);

        Ok(self.try_fetch(id, fragment_count))
    }

    // TODO: Let caller provide output buffer.
    fn try_fetch(&mut self, id: u16, fragment_count: u8) -> Option<Bytes> {
        // establish that there are enough fragments to reconstruct the whole packet
        let payload = {
            let fragments = self.fragment_map.get(&id)?;
            if fragments.len() < fragment_count.into() {
                return None;
            }
            let mut payload =
                BytesMut::with_capacity(fragments.iter().map(|f| f.payload.len()).sum());
            for fragment in fragments {
                payload.extend_from_slice(&fragment.payload);
            }
            payload
        };

        self.fragment_map.remove(&id);
        Some(payload.into())
    }

    pub fn clear_old_fragments(&mut self, max_age: Duration) {
        self.fragment_map.retain(|_, fragments| {
            fragments
                .iter()
                .any(|fragment| fragment.time_received.elapsed() <= max_age)
        });
    }
}

struct Fragment {
    index: u8,
    payload: Bytes,
    time_received: Instant,
}
