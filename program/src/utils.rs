use crate::state::schedule::Schedule;
use unicode_segmentation::UnicodeSegmentation;

pub fn get_domain_price(domain: String, schedule: &Schedule) -> u64 {
    let len = domain.graphemes(true).count() as u64;
    for price in schedule {
        if len == price.length {
            return price.price;
        }
    }
    // Less expensive price
    let last = schedule.last().unwrap();
    last.price
}
