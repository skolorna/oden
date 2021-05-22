import { DateTime } from "luxon";
import { getSkolmatenTimeRanges } from "../../../src/providers/skolmaten/time-range";
import { SkolmatenTimeRange } from "../../../src/providers/skolmaten/types";

test("skolmaten time range conversion", () => {
	expect(getSkolmatenTimeRanges(DateTime.utc(2020), DateTime.utc(2020, 5))).toEqual<SkolmatenTimeRange[]>([
		{
			year: 2020,
			weekOfYear: 1,
			count: 18,
		},
	]);

	expect(getSkolmatenTimeRanges(DateTime.utc(2020, 8), DateTime.utc(2021, 3))).toEqual<SkolmatenTimeRange[]>([
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

	expect(() => getSkolmatenTimeRanges(DateTime.utc(2020), DateTime.utc(2019).minus({ milliseconds: 1 }))).toThrow();
});
