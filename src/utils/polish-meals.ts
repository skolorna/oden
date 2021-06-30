import { Meal } from "../types";
import { trimTitle } from "./trim-title";

/**
 * Deduplicate and trim meals.
 *
 * @param {Meal[]} meals Input meals.
 * @returns {Meal[]} Polished meals.
 */
export function polishMeals(meals: Meal[]): Meal[] {
	const seen: Record<string, boolean> = {};
	const out: Meal[] = [];

	for (let i = 0; i < meals.length; i += 1) {
		const meal = meals[i];

		const polishedValue = trimTitle(meal.value);

		const comp = polishedValue.toLocaleLowerCase();

		if (!seen[comp]) {
			seen[comp] = true;
			out.push({
				// Future proofing; maybe more properties are added to `Meal`.
				...meal,
				value: polishedValue,
			});
		}
	}

	return out;
}
