import { LocalDate } from "js-joda";

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
	date: LocalDate;
	meals: Meal[];
}
