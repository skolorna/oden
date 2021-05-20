import fetch from "node-fetch";
import { URL } from "url";
import { NotFoundError } from "../../errors";
import { GetSchools } from "../types";
import { GetSchoolsResponse, MashieSchool } from "./types";
import { BASE_URL } from "./url";

/**
 * @param query Empty string returns all schools, since the developers behind this "API" are very naïve.
 */
async function fetchSchools(): Promise<GetSchoolsResponse> {
	const data: GetSchoolsResponse = await fetch(new URL("/public/app/internal/execute-query?country=se", BASE_URL), {
		method: "POST",
	}).then((res) => res.json());

	return data;
}

export const listMashieSchools: GetSchools = async () => {
	const schools = await fetchSchools();

	return schools.map(({ id, title }) => ({
		id,
		name: title,
	}));
};

export async function queryMashieSchool(id: string): Promise<MashieSchool> {
	const schools = await fetchSchools();

	const school = schools.find(({ id: schoolID }) => schoolID === id);

	if (!school) {
		throw new NotFoundError(`school with ID \`${id}\` not found!`);
	}

	return school;
}
