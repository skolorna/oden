import { Meal } from "../../src/types";
import { polishMeals } from "../../src/utils/polish-meals";

function generateMeals(values: string[]): Meal[] {
	return values.map((value) => ({ value }));
}

test("polish meals", () => {
	expect(
		polishMeals(
			generateMeals(["Fisk Björkeby", "\n  fisk bJÖRKEBY", "Tacobuffé", " \tPannkaka", "                   Pannkaka"]),
		),
	).toEqual<Meal[]>(generateMeals(["Fisk Björkeby", "Tacobuffé", "Pannkaka"]));
});
