import cheerio, { Element } from "cheerio";
import { LocalDate } from "js-joda";
import fetch from "node-fetch";
import { URL } from "url";
import { ParseError } from "../../errors";
import { Day, Meal } from "../../types";
import { GetMenu } from "../types";
import { getRawMashieSchoolQuerier } from "./schools";
import { MashieGenerator } from "./types";

export const monthLiterals = ["jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec"];

export function parseDateText(input: string): LocalDate {
	const segments = input.split(" ");

	if (segments.length > 3) {
		throw new ParseError(`too many whitespaces in ${input}`);
	}

	const [dayLiteral, monthLiteral, yearLiteral] = segments;

	const day = parseInt(dayLiteral, 10);
	const monthIndex = monthLiterals.indexOf(monthLiteral);
	const year = typeof yearLiteral === "string" ? parseInt(yearLiteral, 10) : new Date().getFullYear();

	if (monthIndex < 0) {
		throw new ParseError(`${monthLiteral} is not a valid month literal`);
	}

	try {
		const date = LocalDate.of(year, monthIndex + 1, day);

		return date;
	} catch (error) {
		throw new ParseError(error.message);
	}
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
		date: parseDateText(dateText),
		meals,
	};
}

export const getMashieMenuGetter: MashieGenerator<GetMenu> = (baseUrl) => {
	const queryMashieSchool = getRawMashieSchoolQuerier(baseUrl);

	return async ({ school, first, last }) => {
		const { url: path } = await queryMashieSchool(school);
		const url = new URL(path, baseUrl);
		const html = await fetch(url).then((res) => res.text());

		const $ = cheerio.load(html);

		const days: Day[] = $(".panel-group > .panel")
			.map((_i, element) => parseDayNode(element))
			.toArray();

		return days.filter(({ date }) => date >= first && date <= last);
	};
};
