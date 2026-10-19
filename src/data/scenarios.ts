import index from "../../data/scenarios/index.json";
import theGathering from "../../data/scenarios/the-gathering.json";
import theMidnightMasks from "../../data/scenarios/the-midnight-masks.json";
import theDevourerBelow from "../../data/scenarios/the-devourer-below.json";

export interface ScenarioTemplate {
  campaign: string;
  code: string;
  name: string;
  order: number;
  chaos_bag: string;
}

const records: Record<string, ScenarioTemplate> = {
  "the-gathering": theGathering,
  "the-midnight-masks": theMidnightMasks,
  "the-devourer-below": theDevourerBelow,
};

export const SCENARIOS: ScenarioTemplate[] = index
  .map((slug) => records[slug])
  .filter((entry): entry is ScenarioTemplate => entry !== undefined);
