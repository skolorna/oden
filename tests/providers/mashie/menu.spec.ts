import cheerio from "cheerio";
import { DateTime } from "luxon";
import { getMashieMenuGetter, monthLiterals, parseDateText, parseMealNode } from "../../../src/providers/mashie/menu";
import { Meal } from "../../../src/types";

describe("mashie menu", () => {
	test("date parsing", () => {
		const { year } = DateTime.now();

		expect(monthLiterals.length).toBe(12);

		expect(parseDateText("17 maj").toISO()).toBe(`${year}-05-17T00:00:00.000+02:00`);
		expect(parseDateText("17 maj 2020").toISODate()).toBe("2020-05-17");
		expect(parseDateText("29 feb 2020").toISODate()).toBe(`2020-02-29`);

		expect(() => parseDateText("May 17")).toThrow();
		expect(() => parseDateText("2020-05-17T00:00:00.000+02:00")).toThrow();
		expect(() => parseDateText("17 maj INVALIDYEAR")).toThrow();
		expect(() => parseDateText("1 december 100 f.Kr.")).toThrow();
		expect(() => parseDateText("29 feb 2021")).toThrow(); // Feb 29, 2021 doesn't exist.
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
			first: DateTime.utc(2000),
			last: DateTime.utc(2077),
		});

		expect(menu.length).toBeGreaterThan(0);
	});
});
