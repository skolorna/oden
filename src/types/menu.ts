import { DateTime } from "luxon";
import Meal from "./meal";

export default interface Menu {
  timestamp: DateTime;
  meals: Meal[];
}
