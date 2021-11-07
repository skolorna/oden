use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    menus::{meal::Meal},
    util::retain_unique,
};

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
            retain_unique(&mut meals);

            Some(Self { date, meals })
        }
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn meals(&self) -> &Vec<Meal> {
        &self.meals
    }

    /// Check if a day is *between* two `NaiveDate`s (inclusive).
    /// ```
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    /// use butler_lib::menus::{day::Day, meal::Meal};
    ///
    /// let meals = vec![Meal::from_str("Sushi").unwrap()];
    /// let day = Day::new_opt(NaiveDate::from_ymd(1789, 7, 14), meals).unwrap();
    ///
    /// assert!(day.is_between(NaiveDate::from_ymd(1789, 7, 10), NaiveDate::from_ymd(1789, 7, 14)));
    /// assert!(!day.is_between(NaiveDate::from_ymd(2020, 5, 4), NaiveDate::from_ymd(2020, 7, 14)));
    /// ```
    /// # Panics
    /// Panics if `lower > upper` in debug mode.
    pub fn is_between(&self, lower: NaiveDate, upper: NaiveDate) -> bool {
        debug_assert!(lower <= upper);

        self.date >= lower && self.date <= upper
    }
}

/// Remove duplicate dates from a vector.
/// ```
/// use chrono::NaiveDate;
/// use std::str::FromStr;
/// use butler_lib::menus::day::{Day, dedup_dates};
/// use butler_lib::menus::meal::Meal;
///
/// let mut days = vec![
///     Day::new_opt(NaiveDate::from_ymd(1789, 7, 14), vec![Meal::from_str("Tacos").unwrap()]).unwrap(),
///     Day::new_opt(NaiveDate::from_ymd(1789, 7, 14), vec![Meal::from_str("Sushi").unwrap()]).unwrap(),
/// ];
///
/// dedup_dates(&mut days);
///
/// assert_eq!(
/// days,
/// [
///     Day::new_opt(NaiveDate::from_ymd(1789, 7, 14), vec![Meal::from_str("Tacos").unwrap()]).unwrap(),
/// ]
/// );
/// ```
pub fn dedup_day_dates(days: &mut Vec<Day>) {
    let mut seen_dates = HashSet::<NaiveDate>::new();
    days.retain(|day| seen_dates.insert(day.date));
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn construct_day() {
        let date = NaiveDate::from_ymd(1789, 7, 14);

        assert!(Day::new_opt(date, vec![]).is_none());
        assert!(Day::new_opt(date, vec![Meal::from_str("Fisk Björkeby").unwrap()]).is_some());
    }

    #[test]
    fn dedup() {
        let mut days = vec![
            Day {
                date: NaiveDate::from_ymd(1789, 7, 14),
                meals: vec![Meal::from_str("Tacos").unwrap()],
            },
            Day {
                date: NaiveDate::from_ymd(1789, 7, 14),
                meals: vec![Meal::from_str("Sushi").unwrap()],
            },
            Day {
                date: NaiveDate::from_ymd(1790, 7, 14),
                meals: vec![Meal::from_str("Pizza").unwrap()],
            },
        ];

        dedup_day_dates(&mut days);

        assert_eq!(
            days,
            vec![
                Day {
                    date: NaiveDate::from_ymd(1789, 7, 14),
                    meals: vec![Meal::from_str("Tacos").unwrap()],
                },
                Day {
                    date: NaiveDate::from_ymd(1790, 7, 14),
                    meals: vec![Meal::from_str("Pizza").unwrap()],
                },
            ]
        )
    }
}
