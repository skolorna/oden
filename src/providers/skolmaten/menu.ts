import { DateTime } from "luxon";
import { URLSearchParams } from "url";
import { Day } from "../../types";
import { GetMenu } from "../types";
import { toSkolmatenID } from "./parser";
import performSkolmatenRequest from "./request";
import { MenuResponse } from "./types";

export interface GetRawMenusOptions extends Record<string, number | undefined> {
	school: number;
	offset?: number;
	limit?: number;
	year?: number;
	week?: number;
}

export async function getRawMenu({
	school,
	offset = 0,
	limit = 1,
	year,
	week,
}: GetRawMenusOptions): Promise<MenuResponse> {
	const params = new URLSearchParams({
		school: school?.toString(),
		offset: offset?.toString(),
		limit: limit?.toString(),
		year: year?.toString(),
		week: week?.toString(),
	});

	return performSkolmatenRequest<MenuResponse>(`/menu?${params.toString()}`);
}

export const getSkolmatenMenu: GetMenu = async ({ school, first = DateTime.now(), last }) => {
	const skolmatenSchool = toSkolmatenID(school);

	const start = first.startOf("day");
	const offset = Math.floor(start.diffNow().as("days"));
	const limit = (last ?? first.plus({ weeks: 1 })).startOf("day").diff(start).as("weeks");

	const res = await getRawMenu({
		school: skolmatenSchool,
		offset,
		limit: Math.ceil(limit),
	});

	const days = res.weeks
		.flatMap((week) => week.days)
		.reduce((acc, { date, meals }) => {
			if (meals && meals.length > 0) {
				const day: Day = {
					timestamp: DateTime.fromMillis(date * 1000),
					meals: meals.map((meal) => ({
						value: meal.value,
					})),
				};

				acc.push(day);
			}

			return acc;
		}, [] as Day[]);

	return days;
};
