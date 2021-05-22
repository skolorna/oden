import { LocalDate } from "js-joda";
import { getSkolmatenTimeRanges } from "../../../src/providers/skolmaten/time-range";
import { SkolmatenTimeRange } from "../../../src/providers/skolmaten/types";

test("skolmaten time range conversion", () => {
	expect(getSkolmatenTimeRanges(LocalDate.of(2020, 1, 1), LocalDate.of(2020, 5, 1))).toEqual<SkolmatenTimeRange[]>([
		{
			year: 2020,
			weekOfYear: 1,
			count: 18,
		},
	]);

	expect(getSkolmatenTimeRanges(LocalDate.of(2020, 8, 1), LocalDate.of(2021, 3, 1))).toEqual<SkolmatenTimeRange[]>([
		{
			year: 2020,
			weekOfYear: 31,
			count: 22,
		},
		{
			year: 2021,
			weekOfYear: 1, // Actually, 53 is the correct week number, but Skolmaten.se doesn't give a f**k.
			count: 9,
		},
	]);

	expect(getSkolmatenTimeRanges(LocalDate.of(2020, 1, 1), LocalDate.of(2020, 12, 31))).toEqual<SkolmatenTimeRange[]>([
		{
			year: 2020,
			weekOfYear: 1,
			count: 53,
		},
	]);

	expect(() => getSkolmatenTimeRanges(LocalDate.of(2020, 1, 1), LocalDate.of(2019, 12, 31))).toThrow();
});
