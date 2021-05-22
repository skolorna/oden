import { FastifyPluginCallback } from "fastify";
import { BadRequest } from "http-errors";
import { DateTime } from "luxon";
import { parseISO8601 } from "../utils/parser";
import { GetMenuQuery, GetMenuQueryType, QuerySchoolParams, QuerySchoolParamsType } from "./route-types";
import { Provider } from "./types";

export function generateProviderRoutes({ info, implementation }: Provider): FastifyPluginCallback {
	return async (fastify) => {
		fastify.get("/", async () => {
			return info;
		});

		fastify.get("/schools", async () => {
			const schools = await implementation.listSchools();

			return schools;
		});

		fastify.get<{
			Params: QuerySchoolParamsType;
		}>(
			"/schools/:schoolId",
			{
				schema: {
					params: QuerySchoolParams,
				},
			},
			async (req) => {
				const { schoolId } = req.params;

				const school = await implementation.querySchool(schoolId);

				return school;
			},
		);

		fastify.get<{
			Params: QuerySchoolParamsType;
			Querystring: GetMenuQueryType;
		}>(
			"/schools/:schoolId/menu",
			{
				schema: {
					params: QuerySchoolParams,
					querystring: GetMenuQuery,
				},
			},
			async (req) => {
				const { schoolId } = req.params;
				const { first: firstLiteral, last: lastLiteral } = req.query;

				const first = firstLiteral ? parseISO8601(firstLiteral) : DateTime.now();
				const last = lastLiteral ? parseISO8601(lastLiteral) : undefined;

				if (last && first > last) {
					throw new BadRequest("?first cannot be after ?last");
				}

				const menu = await implementation.getMenu({
					school: schoolId,
					first,
					last,
				});

				return menu;
			},
		);
	};
}
