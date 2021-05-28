import { Meal } from "../types";

export function dedupMeals(meals: Meal[]): Meal[] {
	const seen: Record<string, boolean> = {};
	const out: Meal[] = [];

	for (let i = 0; i < meals.length; i += 1) {
		const meal = meals[i];

		// TODO: This could be a bit more ambitious ...
		const comp = meal.value.trim().toLocaleLowerCase();

		if (!seen[comp]) {
			seen[comp] = true;
			out.push(meal);
		}
	}

	return out;
}
