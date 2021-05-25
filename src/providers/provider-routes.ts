import { FastifyPluginCallback } from "fastify";
import { BadRequest } from "http-errors";
import { LocalDate, ZoneId } from "js-joda";
import { parseISODate } from "../utils/parser";
import { GetMenuQuery, GetMenuQueryType, QuerySchoolParams, QuerySchoolParamsType } from "./route-types";
import { Provider } from "./types";

export function generateProviderRoutes({ info, implementation }: Provider): FastifyPluginCallback {
	return async (fastify) => {
		fastify.get("/", async () => {
			return info;
		});

		fastify.get("/schools", async () => {
			const schools = await implementation.listMenus();

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

				const school = await implementation.queryMenu(schoolId);

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

				const first = firstLiteral ? parseISODate(firstLiteral) : LocalDate.now(ZoneId.UTC);
				const last = lastLiteral ? parseISODate(lastLiteral) : first.plusWeeks(4);

				if (last && first > last) {
					throw new BadRequest("?first cannot be after ?last");
				}

				const menu = await implementation.listDays({
					menu: schoolId,
					first,
					last,
				});

				return menu;
			},
		);
	};
}
