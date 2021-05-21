import { NotFound } from "http-errors";
import { QuerySchool } from "../types";
import { getRawMenu } from "./menu";
import { toSkolmatenID } from "./parser";

export const querySkolmatenSchool: QuerySchool = async (id) => {
	const { school } = await getRawMenu({
		school: toSkolmatenID(id),
		offset: 0,
		limit: 0,
	});

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
