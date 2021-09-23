import {
    Orchestrator,
} from "@holochain/tryorama";
import { ScenarioApi } from "@holochain/tryorama/lib/api";
import { conductorConfig, installation, sleep} from './common'

const orchestrator = new Orchestrator();

module.exports = async (orchestrator) => {

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
      prev_round_hash = signal.data.payload.signal_payload.round_entry_hash_update;
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
    let GAME_CODE = "ABCDE";

    // alice gets a game code
    let game_code_anchor_entry_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "create_game_code_anchor",
      GAME_CODE
    );
    console.log("Alice created the game code: ", game_code_anchor_entry_hash);
    t.ok(game_code_anchor_entry_hash);

    // alice joins with game code
    let game_code_anchor_entry_hash_alice = await alice_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:GAME_CODE, nickname: "Alice"}
    );
    console.log("Alice joined game: ", game_code_anchor_entry_hash_alice);
    t.deepEqual(game_code_anchor_entry_hash, game_code_anchor_entry_hash_alice);

    // bob joins with game code
    let game_code_anchor_entry_hash_bob = await bob_common.cells[0].call(
      ZOME_NAME,
      "join_game_with_code",
      {gamecode:GAME_CODE, nickname: "Bob"}
    );
    console.log("Bob joined game: ", game_code_anchor_entry_hash_bob);
    t.deepEqual(game_code_anchor_entry_hash, game_code_anchor_entry_hash_bob);

    await sleep(5000); // wait until all links have propagated

    let list_of_players = await alice_common.cells[0].call(
      ZOME_NAME,
      "get_players_for_game_code",
      GAME_CODE
    );
    console.log("===========================");
    console.log(list_of_players);
    t.ok(list_of_players);

    //Alice starts a new game (session) with bob and herself
    let session_header_hash = await alice_common.cells[0].call(
      ZOME_NAME,
      "start_game_session_with_code",
      GAME_CODE
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

}