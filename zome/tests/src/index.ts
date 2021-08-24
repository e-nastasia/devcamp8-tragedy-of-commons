import {
  Orchestrator,
  Config,
  InstallAgentsHapps,
  TransportConfigType,
  Player,
} from "@holochain/tryorama";
import { ScenarioApi } from "@holochain/tryorama/lib/api";
import path from "path";

const conductorConfig = Config.gen({});
console.log('==================');
console.log(conductorConfig);
console.log('==================');

// Construct proper paths for your DNAs
const dnaPath = path.join(__dirname, "../../workdir/dna/sample.dna");

const sleep = (ms) => new Promise<void>((resolve) => setTimeout(() => resolve(), ms));

const orchestrator = new Orchestrator();

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
const installation: InstallAgentsHapps = [
  [[dnaPath]],  // agent 0 - happ 0
  [[dnaPath]],  // agent 1 - happ 0
];

orchestrator.registerScenario(
  "start new game session with 2 players",
  async (s: ScenarioApi, t) => {
    // SETUP
    const ZOME_NAME = "tragedy_of_commons";
     
    const [alice, bob] = await s.players([conductorConfig, conductorConfig]);
    const [[alice_common]] = await alice.installAgentsHapps(installation);
    await sleep(2000);
    const [[bob_common]] = await bob.installAgentsHapps(installation);

    await s.shareAllNodes([alice, bob])

    await sleep(1000)


    // SIGNAL HANDLERS
    let prev_round_hash;
    let signalPromiseAlice = new Promise<void>((resolve) => alice.setSignalHandler((signal) => {
      let payload = signal.data.payload
      t.ok(payload);
      console.log("Alice received Signal:", signal.data.payload);
      prev_round_hash = signal.data.payload.signal_payload.previous_round_entry_hash;
      resolve();
    }));
    // .then(function(data) {
    //     tixel: there should be a way to write this test without the sleep(5000) workaround
    //            using chained promises, but I haven't figured out how
    //     console.log("INSIDE PROMISE:",data);
    //     return null;    
    // });
    let signalPromiseBob = new Promise<void>((resolve) => bob.setSignalHandler((signal) => {
      let payload = signal.data.payload
      t.ok(payload);
      console.log("Bob received Signal:");
      resolve();
    }));


    // START GAME
    //Alice starts a new game (session) with bob and herself
    let session_header_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "start_dummy_session",
      [bob_common.agent, alice_common.agent]
    );
    console.log(session_header_hash);
    t.ok(session_header_hash);

    //Ensure every thing is ok
    await signalPromiseAlice;
    await signalPromiseBob;

    await sleep(2000);
    console.log("prev_round_hash", prev_round_hash);

    // ROUND 1
    // Alice makes 1 move
    let game_move_round_1_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 5, previous_round: prev_round_hash},
    );

    // Bob makes 1 move
    let game_move_round_1_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 125, previous_round: prev_round_hash},
    );
    console.log(game_move_round_1_alice);
    t.ok(game_move_round_1_alice);
    console.log(game_move_round_1_bob);
    t.ok(game_move_round_1_bob);
    
    await sleep(2000);

    // NOTE(e-nastasia): checking that GameSession get fns work as expected
    // maybe should be done in a separate test instead of making thins one 
    // a single super test case, but for speed reasons I'm keeping it here for now
    let alice_owned_games = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    t.ok(alice_owned_games.length == 1);

    let bob_owned_games = await bob_common.cells[0].call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    t.ok(bob_owned_games.length == 0);

    let alice_part_games = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_my_played_sessions",
      null
    );
    t.ok(alice_part_games.length == 0);

    let bob_part_games = await bob_common.cells[0].call(
      ZOME_NAME,
      "get_my_played_sessions",
      null
    );
    t.ok(bob_part_games.length == 1);

    let alice_all_games = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_all_my_sessions",
      null
    );
    t.ok(alice_all_games.length == 1);

    let bob_all_games = await bob_common.cells[0].call(
      ZOME_NAME,
      "get_all_my_sessions",
      null
    );
    t.ok(bob_all_games.length == 1);

    let alice_active_games = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_my_active_sessions",
      null
    );
    t.ok(alice_active_games.length == 1);

    let bob_active_games = await bob_common.cells[0].call(
      ZOME_NAME,
      "get_my_active_sessions",
      null
    );
    t.ok(bob_active_games.length == 1);

    // CHECK  TO CLOSE GAME
    let close_game_round_1_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "try_to_close_round",
      prev_round_hash,
    );
  }
);

orchestrator.run();
