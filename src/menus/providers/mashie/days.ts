import cheerio, { Element } from "cheerio";
import { LocalDate } from "js-joda";
import { URL } from "url";
import { ParseError } from "../../../errors";
import { Day, Meal } from "../../../types";
import { polishMeals } from "../../../utils/polish-meals";
import { fetchRetry } from "../../../utils/fetch-retry";
import { ListDays } from "../types";
import { getRawMashieMenuQuerier } from "./menus";
import { MashieFactory } from "./types";

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

	if (value.length <= 0) {
		throw new ParseError("unable to parse meal node");
	}

	return {
		value,
	};
}

export function parseDayNode(element: Element): Day {
	const $ = cheerio(element);

	const dateText = $.find(".panel-heading .pull-right").text();

	const meals = polishMeals(
		$.find(".app-daymenu-name")
			.toArray()
			.reduce((acc, mealNode) => {
				try {
					acc.push(parseMealNode(mealNode));
				} finally {
					// eslint-disable-next-line no-unsafe-finally
					return acc;
				}
			}, [] as Meal[]),
	);

	return {
		date: parseDateText(dateText),
		meals,
	};
}

export const getMashieDayLister: MashieFactory<ListDays> = (options) => {
	const queryMashieMenu = getRawMashieMenuQuerier(options);

	return async ({ menu, first, last }) => {
		const { url: path } = await queryMashieMenu(menu);
		const url = new URL(path, options.baseUrl);
		const html = await fetchRetry(url).then((res) => res.text());

		const $ = cheerio.load(html);

		const days: Day[] = $(".panel-group > .panel")
			.map((_i, element) => parseDayNode(element))
			.toArray();

		return days.filter(({ date }) => date >= first && date <= last);
	};
};
