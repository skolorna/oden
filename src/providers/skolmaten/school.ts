import { NotFound } from "http-errors";
import { DateTime } from "luxon";
import { QuerySchool } from "../types";
import { getRawMenu } from "./menu";
import { toSkolmatenID } from "./parser";
import { getSkolmatenTimeRanges } from "./time-range";

export const querySkolmatenSchool: QuerySchool = async (id) => {
	const now = DateTime.now();

	const { menu } = await getRawMenu({
		...getSkolmatenTimeRanges(now, now.plus({ weeks: 1 }))[0],
		station: toSkolmatenID(id),
	});

	const school = menu?.station;

	if (!school) {
		throw new NotFound(`school with id \`${id}\` not found`);
	}

	return {
		id: school.id.toString(),
		name: school.name,
		district: school.district.name,
		province: school.district.province.name,
	};
};
