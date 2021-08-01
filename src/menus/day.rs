use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::util::assert_unique;

use super::meal::Meal;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    /// Time zones aren't really relevant here.
    pub date: NaiveDate,
    pub meals: Vec<Meal>,
}

impl Day {
    pub fn new_opt(date: NaiveDate, mut meals: Vec<Meal>) -> Option<Self> {
        if meals.is_empty() {
            None
        } else {
            assert_unique(&mut meals);

            Some(Self { date, meals })
        }
    }
}

/// Remove duplicate dates from a vector.
/// ```
/// use chrono::NaiveDate;
/// use menu_proxy::menus::day::{Day, dedup_dates};
/// use menu_proxy::menus::meal::Meal;
///
/// let mut days = vec![
///   Day {
///     date: NaiveDate::from_ymd(1789, 7, 14),
///     meals: vec![Meal {
///         value: "Tacos".to_owned(),
///     }],
/// },
/// Day {
///     date: NaiveDate::from_ymd(1789, 7, 14),
///     meals: vec![Meal {
///         value: "Sushi".to_owned(),
///     }],
/// },
/// ];
///
/// dedup_dates(&mut days);
///
/// assert_eq!(
/// days,
/// [Day {
///     date: NaiveDate::from_ymd(1789, 7, 14),
///     meals: vec![Meal {
///         value: "Tacos".to_owned(),
///     }]
/// }]
/// )
/// ```
pub fn dedup_dates(days: &mut Vec<Day>) {
    let mut seen_dates = HashSet::<NaiveDate>::new();
    days.retain(|day| seen_dates.insert(day.date));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_day() {
        let date = NaiveDate::from_ymd(1789, 7, 14);

        assert!(Day::new_opt(date, vec![]).is_none());
        assert!(Day::new_opt(date, vec![Meal::from_value("Fisk Björkeby").unwrap()]).is_some());
    }

    #[test]
    fn dedup() {
        let mut days = vec![
            Day {
                date: NaiveDate::from_ymd(1789, 7, 14),
                meals: vec![Meal::from_value("Tacos").unwrap()],
            },
            Day {
                date: NaiveDate::from_ymd(1789, 7, 14),
                meals: vec![Meal::from_value("Sushi").unwrap()],
            },
            Day {
                date: NaiveDate::from_ymd(1790, 7, 14),
                meals: vec![Meal::from_value("Pizza").unwrap()],
            },
        ];

        dedup_dates(&mut days);

        assert_eq!(
            days,
            vec![
                Day {
                    date: NaiveDate::from_ymd(1789, 7, 14),
                    meals: vec![Meal::from_value("Tacos").unwrap()],
                },
                Day {
                    date: NaiveDate::from_ymd(1790, 7, 14),
                    meals: vec![Meal::from_value("Pizza").unwrap()],
                },
            ]
        )
    }
}
