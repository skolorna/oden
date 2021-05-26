import { FastifyPluginCallback } from "fastify";
import { LocalDate, ZoneId } from "js-joda";
import { BadRequest } from "http-errors";
import MenuID from "../menu-id";
import { parseISODate } from "../utils/parser";
import { ListDaysQuery, ListDaysQueryType, QueryMenuParams, QueryMenuParamsType } from "./route-types";
import { getProviderByID, listMenus, providers, queryMenu } from "./universal-provider";

export const providerInfo = providers.map((provider) => provider.info);

export const routes: FastifyPluginCallback = async (fastify) => {
	fastify.get("/", async () => {
		const menus = await listMenus();

		return menus;
	});

	fastify.get<{
		Params: QueryMenuParamsType;
	}>(
		"/:menuId",
		{
			schema: {
				params: QueryMenuParams,
			},
		},
		async (req) => {
			const menuId = MenuID.parse(req.params.menuId);

			const menu = await queryMenu(menuId);

			return menu;
		},
	);

	fastify.get<{
		Params: QueryMenuParamsType;
		Querystring: ListDaysQueryType;
	}>(
		"/:menuId/days",
		{
			schema: {
				params: QueryMenuParams,
				querystring: ListDaysQuery,
			},
		},
		async (req) => {
			const menuId = MenuID.parse(req.params.menuId);
			const { first: firstLiteral, last: lastLiteral } = req.query;

			const first = firstLiteral ? parseISODate(firstLiteral) : LocalDate.now(ZoneId.UTC);
			const last = lastLiteral ? parseISODate(lastLiteral) : first.plusWeeks(4);

			if (last && first > last) {
				throw new BadRequest("?first cannot be after ?last");
			}

			const provider = getProviderByID(menuId.provider);

			const days = await provider.implementation.listDays({
				menu: menuId.providedID,
				first,
				last,
			});

			return days;
		},
	);
};
