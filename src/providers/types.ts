import { DateTime } from "luxon";
import { Day, School, SchoolID } from "../types";

export type GetSchools = () => Promise<School[]>;

export interface GetMenuQuery {
	school: SchoolID;
	first?: DateTime;

	/**
	 * How many *calendar* days to fetch. For example, a value of 7 would return the
	 * meals for the next week, not the next week plus two days (since no food is
	 * served on weekends).
	 */
	limit?: number;
}

export type GetMenu = (query: GetMenuQuery) => Promise<Day[]>;

export interface Provider {
	name: string;
	getSchools: GetSchools;
	getMenu: GetMenu;
}
