import index from "../../data/investigators/index.json";
import agnesBaker from "../../data/investigators/agnes-baker.json";
import daisyWalker from "../../data/investigators/daisy-walker.json";
import rolandBanks from "../../data/investigators/roland-banks.json";
import skidsOtoole from "../../data/investigators/skids-otoole.json";
import wendyAdams from "../../data/investigators/wendy-adams.json";

export interface InvestigatorTemplate {
  code: string;
  name: string;
  class: string;
  willpower: number;
  intellect: number;
  combat: number;
  agility: number;
  max_health: number;
  max_sanity: number;
}

const records: Record<string, InvestigatorTemplate> = {
  "agnes-baker": agnesBaker,
  "daisy-walker": daisyWalker,
  "roland-banks": rolandBanks,
  "skids-otoole": skidsOtoole,
  "wendy-adams": wendyAdams,
};

export const INVESTIGATORS: InvestigatorTemplate[] = index
  .map((slug) => records[slug])
  .filter((entry): entry is InvestigatorTemplate => entry !== undefined);
