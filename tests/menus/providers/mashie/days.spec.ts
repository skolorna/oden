import cheerio from "cheerio";
import { LocalDate } from "js-joda";
import { ParseError } from "../../../../src/errors";
import { generateMashieProvider } from "../../../../src/menus/providers/mashie";
import { monthLiterals, parseDateText, parseDayNode, parseMealNode } from "../../../../src/menus/providers/mashie/days";
import { Day, Meal } from "../../../../src/types";

const provider = generateMashieProvider({
	info: {
		name: "My Provider",
		id: "my-mashie-provider",
	},
	baseUrl: "https://sodexo.mashie.com",
});

describe("mashie days", () => {
	test("date parsing", () => {
		const year = new Date().getFullYear();

		expect(monthLiterals.length).toBe(12);

		expect(parseDateText("17 maj").toString()).toBe(`${year}-05-17`);
		expect(parseDateText("17 maj 2020").toString()).toBe("2020-05-17");
		expect(parseDateText("29 feb 2020").toString()).toBe(`2020-02-29`);

		expect(() => parseDateText("May 17")).toThrowError(ParseError);
		expect(() => parseDateText("2020-05-17T00:00:00.000+02:00")).toThrowError(ParseError);
		expect(() => parseDateText("17 maj INVALIDYEAR")).toThrowError(ParseError);
		expect(() => parseDateText("1 december 100 f.Kr.")).toThrowError(ParseError);
		expect(() => parseDateText("29 feb 2021")).toThrowError(ParseError); // 2021 is not a leap year.
	});

	test("meal node parsing", () => {
		expect(() => parseMealNode(cheerio.load("<div />")("div")[0])).toThrow();
		expect(parseMealNode(cheerio.load("<div>Fisk Björkeby</div>")("div")[0])).toEqual<Meal>({
			value: "Fisk Björkeby",
		});
	});

	test("day node parsing", () => {
		const html = `<div class="day">
			<h4 class="panel-heading">
				<span class="pull-right">17 maj</span>
			</h4>
			<ul>
				<li class="app-daymenu-name">Fisk Björkeby</li>
				<li class="app-daymenu-name">Fisk Björkeby</li>
				<li class="app-daymenu-name">Tacobuffé</li>
			</ul>
		</div>`;

		expect(parseDayNode(cheerio.load(html)(".day")[0])).toEqual<Day>({
			date: LocalDate.of(new Date().getFullYear(), 5, 17),
			meals: [
				{
					value: "Fisk Björkeby",
				},
				{
					value: "Tacobuffé",
				},
			],
		});
	});

	it("should work as intended", async () => {
		const days = await provider.implementation.listDays({
			menu: "b4639689-60f2-4a19-a2dc-abe500a08e45",
			first: LocalDate.of(2000, 1, 1),
			last: LocalDate.of(2077, 1, 1),
		});

		expect(days.length).toBeGreaterThan(0);
	});
});
