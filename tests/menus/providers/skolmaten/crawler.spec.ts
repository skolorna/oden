import { listSkolmatenMenus, validateMenuName } from "../../../../src/menus/providers/skolmaten/crawler";

test("menu name validation", () => {
	expect(validateMenuName("Information!")).toBeFalsy();
	expect(validateMenuName("Södra Latin")).toBeTruthy();
});

test("crawler", async () => {
	jest.setTimeout(20000);

	const menus = await listSkolmatenMenus();

	expect(menus.length).toBeGreaterThan(1000);
});
