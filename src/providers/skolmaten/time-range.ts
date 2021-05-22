import { BadRequest } from "http-errors";
import { DateTime } from "luxon";
import { SkolmatenTimeRange } from "./types";
import { SKOLMATEN_TZ } from "./tz";

/**
 * Skolmaten.se wants queries in years and week numbers.
 *
 * @param {DateTime} start Start time.
 * @param {DateTime} end End time.
 *
 * @returns {SkolmatenTimeRange[]} One or more ranges (if `end` is in another year than `start`).
 */
export function getSkolmatenTimeRanges(start: DateTime, end: DateTime): SkolmatenTimeRange[] {
	if (start > end) {
		throw new BadRequest("start cannot be before end (limit must be non-negative)");
	}

	let localStart = start.setZone(SKOLMATEN_TZ);
	const localEnd = end.setZone(SKOLMATEN_TZ);

	const result: SkolmatenTimeRange[] = [];

	while (localEnd > localStart) {
		let segmentEnd = localEnd;

		if (segmentEnd.year !== localStart.year) {
			segmentEnd = localStart.endOf("year");
		}

		const count = Math.ceil(segmentEnd.diff(localStart).as("weeks"));

		/**
		 * Week number, but never 53.
		 * Skolmaten.se (and I) both dislike when the week number is 53.
		 */
		const weekOfYear = localStart.weekNumber > 52 ? 1 : localStart.weekNumber;

		result.push({
			year: localStart.year,
			weekOfYear,
			count,
		});

		localStart = localStart.plus({ years: 1 }).startOf("year");
	}

	return result;
}
