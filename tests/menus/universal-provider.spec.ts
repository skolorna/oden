import { NotFound } from "http-errors";
import MenuID from "../../src/menu-id";
import { queryMenu } from "../../src/menus/universal-provider";

describe("universal provider", () => {
	test("menu querying", async () => {
		await expect(queryMenu(new MenuID("invalid-provider", "aaa-bbb"))).rejects.toThrowError(NotFound);

		const menu = await queryMenu(new MenuID("sodexo", "b4639689-60f2-4a19-a2dc-abe500a08e45"));

		expect(menu.id.provider).toBe("sodexo");
		expect(menu.id.providedID).toBe("b4639689-60f2-4a19-a2dc-abe500a08e45");
		expect(menu.title).toMatch(/Norra Real/i);
	});
});
