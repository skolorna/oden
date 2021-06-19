import { Meal } from "../../src/types";
import { polishMeals, polishMealValue } from "../../src/utils/polish-meals";

function generateMeals(values: string[]): Meal[] {
	return values.map((value) => ({ value }));
}

test("polish meal names", () => {
	const cases: Record<string, string> = {
		"\t    Fisk Björkeby                   \n": "Fisk Björkeby",
		"\n\n\nVeg. Potatisbullar.\n\n": "Veg. Potatisbullar",
		"A     \n..............              ": "A",
		Tacos: "Tacos",
	};

	Object.entries(cases).forEach(([input, expected]) => {
		expect(polishMealValue(input)).toBe(expected);
	});
});

test("polish meals", () => {
	expect(
		polishMeals(
			generateMeals(["Fisk Björkeby", "\n  fisk bJÖRKEBY", "Tacobuffé", " \tPannkaka", "                   Pannkaka"]),
		),
	).toEqual<Meal[]>(generateMeals(["Fisk Björkeby", "Tacobuffé", "Pannkaka"]));
});
