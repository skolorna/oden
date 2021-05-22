import cheerio from "cheerio";
import { LocalDate } from "js-joda";
import { ParseError } from "../../../src/errors";
import { getMashieMenuGetter, monthLiterals, parseDateText, parseMealNode } from "../../../src/providers/mashie/menu";
import { Meal } from "../../../src/types";

describe("mashie menu", () => {
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

	it("should work as intended", async () => {
		const getMashieMenus = getMashieMenuGetter("https://sodexo.mashie.com");

		const menu = await getMashieMenus({
			school: "b4639689-60f2-4a19-a2dc-abe500a08e45",
			first: LocalDate.of(2000, 1, 1),
			last: LocalDate.of(2077, 1, 1),
		});

		expect(menu.length).toBeGreaterThan(0);
	});
});
