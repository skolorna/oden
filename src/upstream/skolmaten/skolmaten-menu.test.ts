import { DateTime } from "luxon";
import { getSkolmatenMenu } from "./skolmaten-menu";

test("menu", async () => {
	const menu = await getSkolmatenMenu({
		school: "85957002",
		first: DateTime.now(),
		limit: 7,
	});

	expect(menu.length).toBeGreaterThan(0);
	expect(menu.length).toBeLessThanOrEqual(7);

	await expect(
		getSkolmatenMenu({
			school: "invalid-id-because-they-want-to-use-integers-for-some-stupid-reason",
		}),
	).rejects.toThrow();
});
