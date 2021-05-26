import { NotFound } from "http-errors";
import { LocalDate } from "js-joda";
import { QueryMenu } from "../types";
import { getRawDays } from "./days";
import { toSkolmatenID } from "./parser";
import { getSkolmatenTimeRanges } from "./time-range";

export const querySkolmatenMenu: QueryMenu = async (id) => {
	const now = LocalDate.now();

	const { menu } = await getRawDays({
		...getSkolmatenTimeRanges(now, now.plusWeeks(1))[0],
		station: toSkolmatenID(id),
	});

	const station = menu?.station;

	if (!station) {
		throw new NotFound(`menu with id \`${id}\` not found`);
	}

	return {
		id: station.id.toString(),
		title: station.name,
	};
};
