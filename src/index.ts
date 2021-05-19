import { getSkolmatenSchools } from "./providers/skolmaten/crawler";

getSkolmatenSchools().then((schools) => {
	console.log(schools);
});
