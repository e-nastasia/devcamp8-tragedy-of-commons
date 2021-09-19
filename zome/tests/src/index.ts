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
const dnaPath = path.join(__dirname, "../../workdir/dna/tragedy_of_commons.dna");

const sleep = (ms) => new Promise<void>((resolve) => setTimeout(() => resolve(), ms));

const orchestrator = new Orchestrator();

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
const installation: InstallAgentsHapps = [
  [[dnaPath]],  // agent 0 - happ 0
  [[dnaPath]],  // agent 1 - happ 0
];



orchestrator.registerScenario(
  "2 players lose in the first round",
  async (s: ScenarioApi, t) => {
    // SETUP
    const ZOME_NAME = "tragedy_of_commons";
     
    const [alice, bob] = await s.players([conductorConfig, conductorConfig]);
    const [[alice_common]] = await alice.installAgentsHapps(installation);
    await sleep(2000);
    const [[bob_common]] = await bob.installAgentsHapps(installation);

    await s.shareAllNodes([alice, bob])

    await sleep(1000)
    console.debug('***********');
    console.log(bob);
    console.log(bob_common);

    // SIGNAL HANDLERS
    let prev_round_hash;
    let signalPromiseAlice = new Promise<void>((resolve) => alice.setSignalHandler((signal) => {
      let payload = signal.data.payload
      t.ok(payload);
      console.log("Alice received Signal:", signal.data.payload);
      prev_round_hash = signal.data.payload.signal_payload.round_header_hash_update;
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
      console.log("Bob received Signal: {}", payload);
      resolve();
    }));


    // START GAME

    // alice gets a game code
    let game_code_anchor_entry_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "create_game_code_anchor",
      "ABCDE"
    );
    console.log("Alice created the game code: ", game_code_anchor_entry_hash);
    t.ok(game_code_anchor_entry_hash);

    // alice joins with game code
    let game_code_anchor_entry_hash_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:"ABCDE", nickname: "Alice"}
    );
    console.log("Alice joined game: ", game_code_anchor_entry_hash_alice);
    t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_alice);

    // bob joins with game code
    let game_code_anchor_entry_hash_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:"ABCDE", nickname: "Bob"}
    );
    console.log("Bob joined game: ", game_code_anchor_entry_hash_bob);
    t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_bob);

    await sleep(5000); // wait until all links have propagated

    let list_of_players = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_players_for_game_code",
      "ABCDE"
    );
    console.log("===========================");
    console.log(list_of_players);
    t.ok(list_of_players);

    //Alice starts a new game (session) with bob and herself
    let session_header_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "start_game_session_with_code",
      "ABCDE"
    );
    console.log(session_header_hash);
    t.ok(session_header_hash);

    //Ensure every thing is ok
    await signalPromiseAlice;
    await signalPromiseBob;

    await sleep(5000);
    console.log("prev_round_hash", prev_round_hash);

    // ROUND 1
    // Alice makes 1 move
    let game_move_round_1_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 5, previous_round: prev_round_hash},
    );
    
    // test VALIDATION no negative resource amounts in moves
    // let game_move_round_1_alice = await alice_common.cells[0].call(
    //   ZOME_NAME,
    //   "make_new_move",
    //   {resource_amount: -50, previous_round: prev_round_hash},
    // );


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
    // maybe should be done in a separate test instead of making this one
    // a single super test case, but for speed reasons I'm keeping it here for now
    let alice_owned_games = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    console.log("Verify that Alice's owned games is 1");
    t.ok(alice_owned_games.length == 1);

    let bob_owned_games = await bob_common.cells[0].call(
      ZOME_NAME,
      "get_my_owned_sessions",
      null
    );
    console.log("Verify that Bob's owned games is 0");
    t.ok(bob_owned_games.length == 0);

    // CHECK  TO CLOSE GAME
    let close_game_round_1_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "try_to_close_round",
      prev_round_hash,
    );
    console.log("Bob tried to close round 1: ", close_game_round_1_bob);
    console.log("Verify that game has ended and next_action = SHOW_GAME_RESULTS");
    t.ok(close_game_round_1_bob.next_action = "SHOW_GAME_RESULTS");
  }
);

orchestrator.registerScenario(
  "2 players play full game",
  async (s: ScenarioApi, t) => {
    // SETUP
    const ZOME_NAME = "tragedy_of_commons";
     
    const [alice, bob] = await s.players([conductorConfig, conductorConfig]);
    const [[alice_common]] = await alice.installAgentsHapps(installation);
    await sleep(2000);
    const [[bob_common]] = await bob.installAgentsHapps(installation);

    await s.shareAllNodes([alice, bob])

    await sleep(1000)
    console.debug('***********');

    // SIGNAL HANDLERS
    let prev_round_hash;
    let signalPromiseAlice = new Promise<void>((resolve) => alice.setSignalHandler((signal) => {
      let payload = signal.data.payload
      t.ok(payload);
      console.log("Alice received Signal:", signal.data.payload);
      prev_round_hash = signal.data.payload.signal_payload.round_header_hash_update;
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
      console.log("Bob received Signal: {}", payload);
      resolve();
    }));


    // START GAME

    // alice gets a game code
    let game_code_anchor_entry_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "create_game_code_anchor",
      "ABCDE"
    );
    console.log("Alice created the game code: ", game_code_anchor_entry_hash);
    t.ok(game_code_anchor_entry_hash);

    // alice joins with game code
    let game_code_anchor_entry_hash_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:"ABCDE", nickname: "Alice"}
    );
    console.log("Alice joined game: ", game_code_anchor_entry_hash_alice);
    t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_alice);

    // bob joins with game code
    let game_code_anchor_entry_hash_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:"ABCDE", nickname: "Bob"}
    );
    console.log("Bob joined game: ", game_code_anchor_entry_hash_bob);
    t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_bob);

    await sleep(5000); // wait until all links have propagated

    let list_of_players = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_players_for_game_code",
      "ABCDE"
    );
    console.log("===========================");
    console.log(list_of_players);
    t.ok(list_of_players);

    //Alice starts a new game (session) with bob and herself
    let session_header_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "start_game_session_with_code",
      "ABCDE"
    );
    console.log(session_header_hash);
    t.ok(session_header_hash);

    //Ensure every thing is ok
    await signalPromiseAlice;
    await signalPromiseBob;

    await sleep(5000);
    console.log("prev_round_hash", prev_round_hash);

    // ROUND 1
    // Alice makes her move
    let game_move_round_1_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 5, previous_round: prev_round_hash},
    );
    console.log("ROUND 1: Alice made a move: ", game_move_round_1_alice);
    t.ok(game_move_round_1_alice);

    // Bob makes his move
    let game_move_round_1_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 10, previous_round: prev_round_hash},
    );
    console.log("ROUND 1: Bob made a move: ", game_move_round_1_bob);
    t.ok(game_move_round_1_bob);
    
    // wait for move data to propagate
    await sleep(2000);

    // Check to close the first round
    let close_game_round_1_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "try_to_close_round",
      prev_round_hash,
    );
    console.log("Bob tried to close round 1: ", close_game_round_1_bob);
    console.log("Verify that first round has ended and next_action = START_NEXT_ROUND");
    t.ok(close_game_round_1_bob.next_action = "START_NEXT_ROUND");
    prev_round_hash = close_game_round_1_bob.current_round_header_hash;

    // wait for round data to propagate
    await sleep(2000);

    // ROUND 2
    // Alice makes her move
    let game_move_round_2_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 6, previous_round: prev_round_hash},
    );
    console.log("ROUND 2: Alice made a move: ", game_move_round_2_alice);
    t.ok(game_move_round_1_alice);

    // Bob makes his move
    let game_move_round_2_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 11, previous_round: prev_round_hash},
    );
    console.log("ROUND 2: Bob made a move: ", game_move_round_2_bob);
    t.ok(game_move_round_1_bob);

    // Check to close the round
    let close_game_round_2_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "try_to_close_round",
      prev_round_hash,
    );
    console.log("Alice tried to close round 2: ", close_game_round_2_alice);
    console.log("Verify that round 2 has ended and next_action = START_NEXT_ROUND");
    t.ok(close_game_round_1_bob.next_action = "START_NEXT_ROUND");

    // wait for round data to propagate
    await sleep(2000);

    // ROUND 3
    // Alice makes her move
    let game_move_round_3_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 7, previous_round: prev_round_hash},
    );
    console.log("ROUND 3: Alice made a move: ", game_move_round_2_alice);
    t.ok(game_move_round_1_alice);

    // Bob makes his move
    let game_move_round_3_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "make_new_move",
      {resource_amount: 12, previous_round: prev_round_hash},
    );
    console.log("ROUND 3: Bob made a move: ", game_move_round_2_bob);
    t.ok(game_move_round_1_bob);

    // Check to close the round
    let close_game_round_3_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "try_to_close_round",
      prev_round_hash,
    );
    console.log("Alice tried to close round 3: ", close_game_round_3_alice);
    console.log("Verify that round 3 has ended and next_action = SHOW_GAME_RESULTS");
    t.ok(close_game_round_1_bob.next_action = "SHOW_GAME_RESULTS");
  }
);

orchestrator.run();
