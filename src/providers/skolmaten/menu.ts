import { URLSearchParams } from "url";
import { LocalDate } from "js-joda";
import { Day } from "../../types";
import { GetMenu } from "../types";
import { getSkolmatenTimeRanges } from "./time-range";
import { toSkolmatenID } from "./parser";
import performSkolmatenRequest from "./request";
import { MenuResponse, SkolmatenTimeRange } from "./types";

export interface GetRawMenusOptions extends SkolmatenTimeRange, Record<string, number | undefined> {
	/**
	 * ~~School~~ Station ID, Skolmaten.se style.
	 */
	station: number;
}

/**
 * Return the raw, unvalidated menu. Skolmaten.se has a tendency to disrespect
 * offsets and limits at extreme values, smh.
 *
 * @param {GetRawMenusOptions} options Options.
 *
 * @returns {Promise<MenuResponse>} The raw menu response.
 */
export async function getRawMenu({ station, year, weekOfYear, count }: GetRawMenusOptions): Promise<MenuResponse> {
	const params = new URLSearchParams({
		station: station?.toString(),
		year: year?.toString(),
		weekOfYear: weekOfYear?.toString(),
		count: count?.toString(),
	});

	const res = await performSkolmatenRequest<MenuResponse>(`/menu?${params.toString()}`);

	return res;
}

export const getSkolmatenMenu: GetMenu = async ({ school, first, last }) => {
	const skolmatenSchool = toSkolmatenID(school);

	const ranges = getSkolmatenTimeRanges(first, last);

	const responses = await Promise.all(
		ranges.map((range) =>
			getRawMenu({
				...range,
				station: skolmatenSchool,
			}),
		),
	);

	const weeks = responses.flatMap(({ menu }) => menu.weeks);

	const days = weeks
		.flatMap((week) => week.days)
		.reduce((acc, { year, month, day, meals }) => {
			if (meals && meals.length > 0) {
				const date = LocalDate.of(year, month, day);

				if (date >= first && date <= last) {
					acc.push({
						date,
						meals: meals.map((meal) => ({
							value: meal.value,
						})),
					});
				}
			}

			return acc;
		}, [] as Day[]);

	return days;
};
