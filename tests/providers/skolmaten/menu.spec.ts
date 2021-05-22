import { DateTime } from "luxon";
import { getSkolmatenMenu } from "../../../src/providers/skolmaten/menu";

describe("menu test", () => {
	it("should work", async () => {
		const menu = await getSkolmatenMenu({
			school: "85957002",
			first: DateTime.now(),
		});

		expect(menu.length).toBeGreaterThan(0);
		expect(menu.length).toBeLessThanOrEqual(7);
	});

	it("should not return empty arrays", async () => {
		const menu = await getSkolmatenMenu({
			school: "85957002",
			first: DateTime.utc(2077),
		});

		menu.forEach((day) => {
			expect(day.meals.length).toBeGreaterThan(0);
		});
	});

	it("should not accept funny ids", async () => {
		await expect(
			getSkolmatenMenu({
				school: "invalid-id-because-they-want-to-use-integers-for-some-stupid-reason",
			}),
		).rejects.toThrow();
	});
});
