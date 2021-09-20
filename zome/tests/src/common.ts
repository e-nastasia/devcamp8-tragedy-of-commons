import { Orchestrator, Config, InstallAgentsHapps, InstalledHapp } from '@holochain/tryorama'
import path from "path";

export const conductorConfig = Config.gen({});
console.log('==================');
console.log(conductorConfig);
console.log('==================');

// Construct proper paths for your DNAs
const dnaPath = path.join(__dirname, "../../workdir/dna/tragedy_of_commons.dna");

export const sleep = (ms) => new Promise<void>((resolve) => setTimeout(() => resolve(), ms));

const orchestrator = new Orchestrator();

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
export const installation: InstallAgentsHapps = [
  [[dnaPath]],  // agent 0 - happ 0
  [[dnaPath]],  // agent 1 - happ 0
];