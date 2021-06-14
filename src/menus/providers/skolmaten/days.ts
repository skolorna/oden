import { URLSearchParams } from "url";
import { LocalDate } from "js-joda";
import { Day } from "../../../types";
import { ListDays } from "../types";
import { getSkolmatenTimeRanges } from "./time-range";
import { toSkolmatenID } from "./parser";
import performSkolmatenRequest from "./request";
import { MenuResponse, SkolmatenTimeRange } from "./types";
import { dedupMeals } from "../../../utils/dedup-meals";

export interface GetRawDaysOptions extends SkolmatenTimeRange, Record<string, number | undefined> {
	/**
	 * ~~School~~ Station ID, Skolmaten.se style.
	 */
	station: number;
}

/**
 * Return the raw, unvalidated menu. Skolmaten.se has a tendency to disrespect
 * offsets and limits at extreme values, smh.
 *
 * @param {GetRawDaysOptions} options Options.
 *
 * @returns {Promise<MenuResponse>} The raw menu response.
 */
export async function getRawDays({ station, year, weekOfYear, count }: GetRawDaysOptions): Promise<MenuResponse> {
	const params = new URLSearchParams({
		station: station.toString(),
		year: year.toString(),
		weekOfYear: weekOfYear.toString(),
		count: count.toString(),
	});

	const res = await performSkolmatenRequest<MenuResponse>(`/menu?${params.toString()}`);

	return res;
}

export const listSkolmatenDays: ListDays = async ({ menu: menuId, first, last }) => {
	const station = toSkolmatenID(menuId);

	const ranges = getSkolmatenTimeRanges(first, last);

	const responses = await Promise.all(
		ranges.map((range) =>
			getRawDays({
				...range,
				station,
			}),
		),
	);

	const weeks = responses.flatMap(({ menu }) => menu.weeks);

	const days = weeks
		.flatMap((week) => week.days)
		.reduce((acc, { year, month, day, meals: inputMeals }) => {
			if (inputMeals && inputMeals.length > 0) {
				const date = LocalDate.of(year, month, day);

				if (date >= first && date <= last) {
					const meals = dedupMeals(inputMeals).map(({ value }) => ({ value }));

					acc.push({
						date,
						meals,
					});
				}
			}

			return acc;
		}, [] as Day[]);

	return days;
};
