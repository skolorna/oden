import { NotFound } from "http-errors";
import { generateMashieProvider } from "../../../../src/menus/providers/mashie";

const provider = generateMashieProvider({
	info: {
		id: "my-provider",
		name: "My Provider",
	},
	baseUrl: "https://sodexo.mashie.com",
});

test("list menus", async () => {
	const menus = await provider.implementation.listMenus();

	expect(menus.length).toBeGreaterThan(100);
});

describe("query menus", () => {
	it("should throw an error if no menu is found", async () => {
		await expect(provider.implementation.queryMenu("invalid-id-that-should-not-be-used")).rejects.toThrowError(
			NotFound,
		);
	});

	it("should work as expected", async () => {
		const menu = await provider.implementation.queryMenu("b4639689-60f2-4a19-a2dc-abe500a08e45");

		expect(menu.title).toMatch(/Norra Real/i);
	});
});
