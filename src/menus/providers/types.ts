import { LocalDate } from "js-joda";
import { Day } from "../../types";

export interface ProviderMenu {
	id: string;
	title: string;
}

export type ListMenus = () => Promise<ProviderMenu[]>;

export type QueryMenu = (id: string) => Promise<ProviderMenu>;

export interface ListDaysQuery {
	menu: string;
	first: LocalDate;
	last: LocalDate;
}

export type ListDays = (query: ListDaysQuery) => Promise<Day[]>;

export interface ProviderInfo {
	name: string;
	id: string;
}

export interface ProviderImplementation {
	listMenus: ListMenus;
	queryMenu: QueryMenu;
	listDays: ListDays;
}

export interface Provider {
	info: ProviderInfo;
	implementation: ProviderImplementation;
}
