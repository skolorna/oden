import { DateTime } from "luxon";
import { Day, School, SchoolID } from "../types";

export type GetSchools = () => Promise<School[]>;

export interface GetMenuQuery {
	school: SchoolID;
	first?: DateTime;
	last?: DateTime;
}

export type GetMenu = (query: GetMenuQuery) => Promise<Day[]>;

export interface ProviderInfo {
	name: string;
	id: string;
}

export interface ProviderImplementation {
	getSchools: GetSchools;
	getMenu: GetMenu;
}

export interface Provider {
	info: ProviderInfo;
	implementation: ProviderImplementation;
}
