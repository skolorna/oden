use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{util::retain_unique, Meal};

/// A day is localized to a single menu and contains
/// a list of the meals served there on a particular
/// date.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    /// Date of the day. (Time zones aren't really relevant here.)
    pub date: NaiveDate,

    /// Meals served on this day.
    pub meals: Vec<Meal>,
}

impl Day {
    /// Construct a day, but disallow empty meals.
    ///
    /// ```
    /// # use chrono::NaiveDate;
    /// # use hugin::{Day, Meal};
    /// # use std::str::FromStr;
    /// #
    /// let date = NaiveDate::from_ymd(1789, 7, 14);
    ///
    /// assert!(Day::new_opt(date, vec![]).is_none());
    /// assert!(Day::new_opt(date, vec![Meal::from_str("Fisk Björkeby").unwrap()]).is_some());
    /// ```
    #[must_use]
    pub fn new_opt(date: NaiveDate, mut meals: Vec<Meal>) -> Option<Self> {
        if meals.is_empty() {
            None
        } else {
            retain_unique(&mut meals);

            Some(Self { date, meals })
        }
    }

    /// Date, in any timezone.
    #[must_use]
    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// Get the meals served.
    #[must_use]
    pub fn meals(&self) -> &[Meal] {
        &self.meals
    }

    /// Check if a day is *between* two `NaiveDate`s (inclusive).
    /// ```
    /// use chrono::NaiveDate;
    /// use std::str::FromStr;
    /// use hugin::{Day, Meal};
    ///
    /// let meals = vec![Meal::from_str("Sushi").unwrap()];
    /// let day = Day::new_opt(NaiveDate::from_ymd(1789, 7, 14), meals).unwrap();
    ///
    /// assert!(day.is_between(NaiveDate::from_ymd(1789, 7, 10), NaiveDate::from_ymd(1789, 7, 14)));
    /// assert!(!day.is_between(NaiveDate::from_ymd(2020, 5, 4), NaiveDate::from_ymd(2020, 7, 14)));
    /// ```
    /// # Panics
    /// Panics if `lower > upper` in debug mode.
    #[must_use]
    pub fn is_between(&self, lower: NaiveDate, upper: NaiveDate) -> bool {
        debug_assert!(lower <= upper);

        self.date >= lower && self.date <= upper
    }
}

/// Remove duplicate dates from a vector.
pub(crate) fn dedup_day_dates(days: &mut Vec<Day>) {
    let mut seen_dates = HashSet::<NaiveDate>::new();
    days.retain(|day| seen_dates.insert(day.date));
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use std::str::FromStr;

    use super::*;

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
        );
    }

    #[test]
    fn dedup_dates() {
        let mut days = vec![
            Day::new_opt(
                NaiveDate::from_ymd(1789, 7, 14),
                vec![Meal::from_str("Tacos").unwrap()],
            )
            .unwrap(),
            Day::new_opt(
                NaiveDate::from_ymd(1789, 7, 14),
                vec![Meal::from_str("Sushi").unwrap()],
            )
            .unwrap(),
        ];

        dedup_day_dates(&mut days);

        assert_eq!(
            days,
            [Day::new_opt(
                NaiveDate::from_ymd(1789, 7, 14),
                vec![Meal::from_str("Tacos").unwrap()]
            )
            .unwrap(),]
        );
    }
}
