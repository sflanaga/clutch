use std::time::Duration;
use std::fmt::Display;
use num_format::{Locale, ToFormattedString};
use std::convert::TryInto;
use num_traits::AsPrimitive;

pub fn rate<T: AsPrimitive<f64>>(cnt: T, dur: Duration) -> Box<dyn Display> {
    let cnt_f = cnt.as_();
    let t_secs = dur.as_secs_f64();
    let rate = (cnt_f/t_secs) as usize;
    Box::new(rate.to_formatted_string(&Locale::en))
}

pub fn comma<T: AsPrimitive<u64>>(cnt: T) -> Box<dyn Display> {
    Box::new((cnt.as_()).to_formatted_string(&Locale::en))
}

