import { Meal } from "../types";

export function dedupMeals(meals: Meal[]): Meal[] {
	const seen: Record<string, boolean> = {};
	const out: Meal[] = [];

	for (let i = 0; i < meals.length; i += 1) {
		const meal = meals[i];

		if (!seen[meal.value]) {
			seen[meal.value] = true;
			out.push(meal);
		}
	}

	return out;
}
