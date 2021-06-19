import { Meal } from "../types";

/**
 * Polish a meal by trimming the value.
 *
 * @param {Meal} meal Meal.
 * @returns {Meal} Polished meal.
 */
export function polishMealValue(value: string): string {
	// Trim same characters as String.trim(), as well as punctuation.
	// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/Trim#polyfill
	return value.replace(/^[\s\uFEFF\xA0.]+|[\s\uFEFF\xA0.]+$/g, "");
}

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

		const polishedValue = polishMealValue(meal.value);

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
