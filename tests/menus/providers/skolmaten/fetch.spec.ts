import skolmatenFetch, { getSkolmatenUrl } from "../../../../src/menus/providers/skolmaten/fetch";
import { DistrictsResponse } from "../../../../src/menus/providers/skolmaten/types";

test("skolmaten urls", () => {
	const cases: Record<string, Parameters<typeof getSkolmatenUrl>> = {
		"https://skolmaten.se/api/4/districts": ["districts"],
		"https://skolmaten.se/api/4/stations?district=69420": [
			"stations",
			new URLSearchParams({
				district: "69420",
			}),
		],
		"https://skolmaten.se/api/4/a/b?c=4%26d%3D5": [
			"/a/b/",
			new URLSearchParams({
				c: "4&d=5",
			}),
		],
		"https://skolmaten.se/api/4/provinces": ["provinces"],
	};

	Object.entries(cases).forEach(([expected, input]) => {
		expect(getSkolmatenUrl(...input)).toBe(expected);
	});
});

test("skolmaten api client", async () => {
	const { districts } = await skolmatenFetch<DistrictsResponse>(
		"districts",
		new URLSearchParams({
			province: "5662940552232960",
		}),
	);

	expect(districts.length).toBeGreaterThan(10);
});
