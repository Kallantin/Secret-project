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
  acts: ScenarioActData[];
  agendas: ScenarioAgendaData[];
  locations: ScenarioLocationData[];
}

export interface ScenarioActData {
  code: string;
  name: string;
  text: string;
  clue_cost: number;
  raw: Record<string, unknown>;
}

export interface ScenarioAgendaData {
  code: string;
  name: string;
  text: string;
  doom_threshold: number;
  raw: Record<string, unknown>;
}

export interface ScenarioLocationData {
  code: string;
  name: string;
  text: string;
  clues: number;
  shroud: number;
  raw: Record<string, unknown>;
}

const records: Record<string, ScenarioTemplate> = {
  "the-gathering": theGathering,
  "the-midnight-masks": theMidnightMasks,
  "the-devourer-below": theDevourerBelow,
};

export const SCENARIOS: ScenarioTemplate[] = index
  .map((slug) => records[slug])
  .filter((entry): entry is ScenarioTemplate => entry !== undefined);
