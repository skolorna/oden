#![no_main]
use libfuzzer_sys::{fuzz_target, Corpus};
use hed::Key;

fuzz_target!(|input: (uuid::Uuid, (i32, u16), u32)| -> Corpus {
    let (menu, (year, ord), i) = input;
    let date = match time::Date::from_ordinal_date(year, ord) {
        Ok(d) => d,
        Err(_) => return Corpus::Reject,
    };
    let key = Key::new(menu, date, i);
    assert_eq!(key.menu(), menu);
    assert_eq!(key.date(), Some(date));
    assert_eq!(key.i(), i);
    Corpus::Keep
});
