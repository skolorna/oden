import { LocalDate } from "js-joda";
import MenuID from "./menu-id";

export interface ProviderInfo {
	name: string;
	id: string;
}

export interface Menu {
	id: MenuID;
	title: string;
	provider: ProviderInfo;
}

export interface Meal {
	value: string;
}

export interface Day {
	date: LocalDate;
	meals: Meal[];
}
