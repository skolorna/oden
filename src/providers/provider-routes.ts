import { FastifyPluginCallback } from "fastify";
import { QuerySchoolOptions, QuerySchoolOptionsType } from "./route-types";
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
			Params: QuerySchoolOptionsType;
		}>(
			"/schools/:schoolId",
			{
				schema: {
					params: QuerySchoolOptions,
				},
			},
			async (req) => {
				const { schoolId } = req.params;

				const school = await implementation.querySchool(schoolId);

				return school;
			},
		);
	};
}
