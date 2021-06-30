import { trimTitle } from "../../src/utils/trim-title";

test("trim titles", () => {
	const cases: Record<string, string> = {
		"\t    Fisk Björkeby                   \n": "Fisk Björkeby",
		"\n\n\nVeg. Potatisbullar.\n\n": "Veg. Potatisbullar",
		"A     \n..............              ": "A",
		Tacos: "Tacos",
		"    Konstig                  Skola\t\t": "Konstig Skola",
	};

	Object.entries(cases).forEach(([input, expected]) => {
		expect(trimTitle(input)).toBe(expected);
	});
});
