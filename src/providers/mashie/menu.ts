import cheerio, { Element } from "cheerio";
import { DateTime } from "luxon";
import fetch from "node-fetch";
import { URL } from "url";
import { ParseError } from "../../errors";
import { Day, Meal } from "../../types";
import { GetMenu } from "../types";
import { getRawMashieSchoolQuerier } from "./schools";
import { MashieGenerator } from "./types";
import { MASHIE_TZ } from "./tz";

export const monthLiterals = ["jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec"];

export function parseDateText(input: string): DateTime {
	const segments = input.split(" ");

	if (segments.length > 3) {
		throw new ParseError(`too many whitespaces in ${input}`);
	}

	const [dayLiteral, monthLiteral, yearLiteral] = segments;

	const day = parseInt(dayLiteral, 10);
	const monthIndex = monthLiterals.indexOf(monthLiteral);
	const year = typeof yearLiteral === "string" ? parseInt(yearLiteral, 10) : undefined;

	if (monthIndex < 0) {
		throw new ParseError(`${monthLiteral} is not a valid month literal`);
	}

	const date = DateTime.fromObject({
		day,
		month: monthIndex + 1,
		year,
		zone: MASHIE_TZ,
	});

	if (!date.isValid) {
		throw new ParseError(date.invalidExplanation ?? `cannot parse ${input}`);
	}

	return date;
}

export function parseMealNode(element: Element): Meal {
	const value = cheerio(element).text();

	if (value?.length <= 0) {
		throw new ParseError("unable to parse meal node");
	}

	return {
		value,
	};
}

export function parseDayNode(element: Element): Day {
	const dateText = cheerio(element).find(".panel-heading .pull-right").text();

	const meals: Meal[] = cheerio(element)
		.find(".app-daymenu-name")
		.map((_, mealNode) => parseMealNode(mealNode))
		.toArray();

	return {
		timestamp: parseDateText(dateText),
		meals,
	};
}

export const getMashieMenuGetter: MashieGenerator<GetMenu> = (baseUrl) => {
	const queryMashieSchool = getRawMashieSchoolQuerier(baseUrl);

	return async ({ school, first = DateTime.now(), last }) => {
		const { url: path } = await queryMashieSchool(school);
		const url = new URL(path, baseUrl);
		const html = await fetch(url).then((res) => res.text());

		const $ = cheerio.load(html);

		const days: Day[] = $(".panel-group > .panel")
			.map((_i, element) => parseDayNode(element))
			.toArray();

		const start = first.startOf("day");
		const end = last?.endOf("day");

		return days.filter(({ timestamp }) => {
			if (timestamp < start) {
				return false;
			}

			if (end && timestamp > end) {
				return false;
			}

			return true;
		});
	};
};
