import { DateTime } from "luxon";
import { URLSearchParams } from "url";
import { GetMenu } from "../types";
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

export const getSkolmatenMenu: GetMenu = async ({ school, first = DateTime.now(), limit = 7 }) => {
	const skolmatenSchool = parseInt(school, 10);

	if (!Number.isInteger(skolmatenSchool)) {
		throw new Error("School ID must be an integer!");
	}

	const start = first.startOf("day");
	const offset = Math.floor(start.diffNow().as("days"));

	const res = await getRawMenu({
		school: skolmatenSchool,
		offset,
		limit: Math.ceil(limit / 7),
	});

	const days = res.weeks.flatMap((week) => week.days);

	return days.map((day) => ({
		timestamp: DateTime.fromMillis(day.date),
		meals:
			day.meals?.map((meal) => ({
				value: meal.value,
			})) ?? [],
	}));
};
