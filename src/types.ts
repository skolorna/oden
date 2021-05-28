import { LocalDate } from "js-joda";
import MenuID from "./menu-id";

export interface Menu {
	id: MenuID;
	title: string;
}

export interface Meal {
	value: string;
}

export interface Day {
	date: LocalDate;
	meals: Meal[];
}
