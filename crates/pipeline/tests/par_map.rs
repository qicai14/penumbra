//! T77: `par_map` stops every worker on the first error instead of running the other items out.

use penumbra_pipeline::v2::par_map;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn par_map_stops_picking_up_items_after_the_first_error() {
    let items: Vec<usize> = (0..2_000).collect();
    let calls = AtomicUsize::new(0);
    let r = par_map(&items, |&i| {
        calls.fetch_add(1, Ordering::Relaxed);
        if i == 0 {
            return Err("item 0 failed".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
        Ok(i)
    });
    assert_eq!(r.unwrap_err().to_string(), "item 0 failed");
    let n = calls.load(Ordering::Relaxed);
    assert!(n < items.len() / 4, "{n} of {} items ran", items.len());
}

#[test]
fn par_map_keeps_input_order() {
    let items: Vec<u32> = (0..100).collect();
    let r = par_map(&items, |&i| Ok(i * 2)).unwrap();
    assert_eq!(r, (0..100).map(|i| i * 2).collect::<Vec<_>>());
}
