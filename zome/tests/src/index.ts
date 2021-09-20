import {
  Orchestrator,
} from "@holochain/tryorama";

let orchestrator = new Orchestrator()
require('./two-players-lose-first-round')(orchestrator)
orchestrator.run()

// orchestrator.registerScenario(
//   "2 players play full game",
//   async (s: ScenarioApi, t) => {
//     // SETUP
//     const ZOME_NAME = "tragedy_of_commons";
     
//     const [alice, bob] = await s.players([conductorConfig, conductorConfig]);
//     const [[alice_common]] = await alice.installAgentsHapps(installation);
//     await sleep(2000);
//     const [[bob_common]] = await bob.installAgentsHapps(installation);

//     await s.shareAllNodes([alice, bob])

//     await sleep(1000)
//     console.debug('***********');

//     // SIGNAL HANDLERS
//     let prev_round_hash;
//     let signalPromiseAlice = new Promise<void>((resolve) => alice.setSignalHandler((signal) => {
//       let payload = signal.data.payload
//       t.ok(payload);
//       console.log("Alice received Signal:", signal.data.payload);
//       prev_round_hash = signal.data.payload.signal_payload.round_header_hash_update;
//       resolve();
//     }));
//     // .then(function(data) {
//     //     tixel: there should be a way to write this test without the sleep(5000) workaround
//     //            using chained promises, but I haven't figured out how
//     //     console.log("INSIDE PROMISE:",data);
//     //     return null;    
//     // });
//     let signalPromiseBob = new Promise<void>((resolve) => bob.setSignalHandler((signal) => {
//       let payload = signal.data.payload
//       t.ok(payload);
//       console.log("Bob received Signal: {}", payload);
//       resolve();
//     }));


//     // START GAME
//     let GAME_CODE = "F1F2F3";

//     // alice gets a game code
//     let game_code_anchor_entry_hash = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "create_game_code_anchor",
//       GAME_CODE
//     );
//     console.log("Alice created the game code: ", game_code_anchor_entry_hash);
//     t.ok(game_code_anchor_entry_hash);

//     // alice joins with game code
//     let game_code_anchor_entry_hash_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "join_game_with_code",
//       {gamecode:GAME_CODE, nickname: "Alice"}
//     );
//     console.log("Alice joined game: ", game_code_anchor_entry_hash_alice);
//     t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_alice);

//     // bob joins with game code
//     let game_code_anchor_entry_hash_bob = await bob_common.cells[0].call(
//       ZOME_NAME,
//       "join_game_with_code",
//       {gamecode:GAME_CODE, nickname: "Bob"}
//     );
//     console.log("Bob joined game: ", game_code_anchor_entry_hash_bob);
//     t.equal(game_code_anchor_entry_hash, game_code_anchor_entry_hash_bob);

//     await sleep(5000); // wait until all links have propagated

//     let list_of_players = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "get_players_for_game_code",
//       GAME_CODE
//     );
//     console.log("===========================");
//     console.log(list_of_players);
//     t.ok(list_of_players);

//     //Alice starts a new game (session) with bob and herself
//     let session_header_hash = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "start_game_session_with_code",
//       GAME_CODE
//     );
//     console.log(session_header_hash);
//     t.ok(session_header_hash);

//     //Ensure every thing is ok
//     await signalPromiseAlice;
//     await signalPromiseBob;

//     await sleep(5000);
//     console.log("prev_round_hash", prev_round_hash);

//     // ROUND 1
//     // Alice makes her move
//     let game_move_round_1_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 5, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 1: Alice made a move: ", game_move_round_1_alice);
//     t.ok(game_move_round_1_alice);

//     // Bob makes his move
//     let game_move_round_1_bob = await bob_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 10, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 1: Bob made a move: ", game_move_round_1_bob);
//     t.ok(game_move_round_1_bob);
    
//     // wait for move data to propagate
//     await sleep(2000);

//     // Check to close the first round
//     let close_game_round_1_bob = await bob_common.cells[0].call(
//       ZOME_NAME,
//       "try_to_close_round",
//       prev_round_hash,
//     );
//     console.log("Bob tried to close round 1: ", close_game_round_1_bob);
//     console.log("Verify that first round has ended and next_action = START_NEXT_ROUND");
//     t.ok(close_game_round_1_bob.next_action = "START_NEXT_ROUND");
//     prev_round_hash = close_game_round_1_bob.current_round_header_hash;

//     // wait for round data to propagate
//     await sleep(2000);

//     // ROUND 2
//     // Alice makes her move
//     let game_move_round_2_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 6, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 2: Alice made a move: ", game_move_round_2_alice);
//     t.ok(game_move_round_1_alice);

//     // Bob makes his move
//     let game_move_round_2_bob = await bob_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 11, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 2: Bob made a move: ", game_move_round_2_bob);
//     t.ok(game_move_round_1_bob);

//     // Check to close the round
//     let close_game_round_2_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "try_to_close_round",
//       prev_round_hash,
//     );
//     console.log("Alice tried to close round 2: ", close_game_round_2_alice);
//     console.log("Verify that round 2 has ended and next_action = START_NEXT_ROUND");
//     t.ok(close_game_round_1_bob.next_action = "START_NEXT_ROUND");

//     // wait for round data to propagate
//     await sleep(2000);

//     // ROUND 3
//     // Alice makes her move
//     let game_move_round_3_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 7, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 3: Alice made a move: ", game_move_round_2_alice);
//     t.ok(game_move_round_1_alice);

//     // Bob makes his move
//     let game_move_round_3_bob = await bob_common.cells[0].call(
//       ZOME_NAME,
//       "make_new_move",
//       {resource_amount: 12, previous_round: prev_round_hash},
//     );
//     console.log("ROUND 3: Bob made a move: ", game_move_round_2_bob);
//     t.ok(game_move_round_1_bob);

//     // Check to close the round
//     let close_game_round_3_alice = await alice_common.cells[0].call(
//       ZOME_NAME,
//       "try_to_close_round",
//       prev_round_hash,
//     );
//     console.log("Alice tried to close round 3: ", close_game_round_3_alice);
//     console.log("Verify that round 3 has ended and next_action = SHOW_GAME_RESULTS");
//     t.ok(close_game_round_1_bob.next_action = "SHOW_GAME_RESULTS");
//   }
// );

