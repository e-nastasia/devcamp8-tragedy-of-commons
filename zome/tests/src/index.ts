import {
  Orchestrator,
} from "@holochain/tryorama";

let orchestrator = new Orchestrator()
require('./two-players-lose-first-round')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./two-players-full-game-finished')(orchestrator)
orchestrator.run()
