import { Meal } from "../../src/types";
import { dedupMeals } from "../../src/utils/dedup-meals";

function generateMeals(values: string[]): Meal[] {
	return values.map((value) => ({ value }));
}

test("dedup meals", () => {
	expect(dedupMeals(generateMeals(["Fisk Björkeby", "\n  fisk bJÖRKEBY", "Tacobuffé"]))).toEqual<Meal[]>(
		generateMeals(["Fisk Björkeby", "Tacobuffé"]),
	);
});
