import { DateTime } from "luxon";
import { getSkolmatenMenu } from "../../../src/providers/skolmaten/menu";

describe("menu test", () => {
	it("should work", async () => {
		const first = DateTime.utc(2021, 6, 1);
		const last = first.plus({ weeks: 1 }).endOf("day");

		const menu = await getSkolmatenMenu({
			school: "85957002",
			first,
			last,
		});

		expect(menu.length).toBeGreaterThan(0);
		expect(menu.length).toBeLessThanOrEqual(7);
	});

	it("should not return empty arrays", async () => {
		const menu = await getSkolmatenMenu({
			school: "85957002",
			first: DateTime.utc(2077),
			last: DateTime.utc(2079),
		});

		menu.forEach((day) => {
			expect(day.meals.length).toBeGreaterThan(0);
		});
	});

	it("should not accept funny ids", async () => {
		await expect(
			getSkolmatenMenu({
				school: "invalid-id-because-they-want-to-use-integers-for-some-stupid-reason",
				first: DateTime.now(),
				last: DateTime.now(),
			}),
		).rejects.toThrow();
	});
});
