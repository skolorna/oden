import { BadRequest } from "http-errors";
import { ChronoUnit, LocalDate } from "js-joda";
import { SkolmatenTimeRange } from "./types";

/**
 * Skolmaten.se wants queries in years and week numbers.
 *
 * @param {LocalDate} start Start date.
 * @param {LocalDate} end End date.
 *
 * @returns {SkolmatenTimeRange[]} One or more ranges (if `end` is in another year than `start`).
 */
export function getSkolmatenTimeRanges(start: LocalDate, end: LocalDate): SkolmatenTimeRange[] {
	if (start > end) {
		throw new BadRequest("start cannot be before end (limit must be non-negative)");
	}

	let segmentStart = start;

	const result: SkolmatenTimeRange[] = [];

	while (end > segmentStart) {
		let segmentEnd = end;

		if (segmentEnd.year() !== segmentStart.year()) {
			segmentEnd = segmentStart.withMonth(12).withDayOfMonth(31);
		}

		const count = Math.ceil(segmentStart.until(segmentEnd, ChronoUnit.DAYS) / 7) + 1;

		/**
		 * Week number, but never 53.
		 * Skolmaten.se (and I) both dislike when the week number is 53.
		 */
		let weekOfYear = segmentStart.isoWeekOfWeekyear();

		if (weekOfYear > 52) {
			weekOfYear = 1;
		}

		result.push({
			year: segmentStart.year(),
			weekOfYear,
			count,
		});

		segmentStart = segmentStart.plusYears(1).withDayOfYear(1);
	}

	return result;
}
