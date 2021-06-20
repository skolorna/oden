import { listSkolmatenMenus, menuNameIsValid } from "../../../../src/menus/providers/skolmaten/crawler";

test("menu name validation", () => {
	expect(menuNameIsValid("Information!")).toBeFalsy();
	expect(menuNameIsValid("Södra Latin")).toBeTruthy();
});

test("crawler", async () => {
	jest.setTimeout(20000);

	const menus = await listSkolmatenMenus();

	expect(menus.length).toBeGreaterThan(1000);
});
