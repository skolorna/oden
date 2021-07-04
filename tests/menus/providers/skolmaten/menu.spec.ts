import { NotFound } from "http-errors";
import { querySkolmatenMenu } from "../../../../src/menus/providers/skolmaten/menu";

test("skolmaten menu", async () => {
	const menu = await querySkolmatenMenu("85957002");

	expect(menu.title).toMatchInlineSnapshot(`"P A Fogelströms gymnasium, Stockholms stad"`);

	await expect(querySkolmatenMenu("a")).rejects.toThrow();
	await expect(querySkolmatenMenu("123")).rejects.toThrowError(NotFound);
});
