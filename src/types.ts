import { DateTime } from "luxon";

export type SchoolID = string;

export interface School {
	id: SchoolID;
	name: string;
	province?: string;
	district?: string;
}

export interface Meal {
	value: string;
}

export interface Day {
	timestamp: DateTime;
	meals: Meal[];
}
