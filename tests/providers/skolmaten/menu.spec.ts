import { LocalDate } from "js-joda";
import { getSkolmatenMenu } from "../../../src/providers/skolmaten/menu";

describe("menu test", () => {
	it("should work", async () => {
		const first = LocalDate.of(2021, 6, 1);
		const last = first.plusWeeks(1);

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
			first: LocalDate.of(2077, 1, 1),
			last: LocalDate.of(2079, 1, 1),
		});

		menu.forEach((day) => {
			expect(day.meals.length).toBeGreaterThan(0);
		});
	});

	it("should not accept funny ids", async () => {
		await expect(
			getSkolmatenMenu({
				school: "invalid-id-because-they-want-to-use-integers-for-some-stupid-reason",
				first: LocalDate.now(),
				last: LocalDate.now(),
			}),
		).rejects.toThrow();
	});
});
