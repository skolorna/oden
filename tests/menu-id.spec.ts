import MenuID from "../src/menu-id";

describe("MenuID", () => {
	it("serializes correctly", () => {
		const id = new MenuID("sodexo", "aaa-bbb-ccc-ddd");

		expect(id.toString()).toBe("sodexo.aaa-bbb-ccc-ddd");
		expect(JSON.stringify(id)).toBe('"sodexo.aaa-bbb-ccc-ddd"');
		expect(JSON.stringify({ id })).toBe('{"id":"sodexo.aaa-bbb-ccc-ddd"}');
	});

	it("parses correctly", () => {
		expect(MenuID.parse("sodexo.aaa-bbb-ccc-ddd").provider).toBe("sodexo");
		expect(MenuID.parse("sodexo.aaa-bbb-ccc-ddd").providedID).toBe("aaa-bbb-ccc-ddd");
		expect(() => MenuID.parse("invalid")).toThrow();
	});
});
