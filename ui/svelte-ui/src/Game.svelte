<script>
    import GameMove from "./GameMove.svelte";
    import GameResults from "./GameResults.svelte";
    import GameRound from "./GameRound.svelte";
    import { bufferToBase64 } from "./utils";

    export let nickname = "Tixel";
    export let gamecode = "3KL54M";
    export let action = "GAME_BEGIN";

    const DELAY = 300;
    let game_status = "WAITING_PLAYERS"; // "MAKE_MOVE" "WAIT_NEXT_ROUND" "WAIT_GAME_SCORE" "GAME_OVER"
    let result_status = "WAIT_RESULTS"; //"GAME_LOST" "GAME_WON"

    async function refreshPlayerList() {
        const playerProfiles = await window.appClient.getPlayers(gamecode);
        console.log("players", playerProfiles);
        players = playerProfiles;
    }

    async function play() {
        console.log("action", action);
        if (action == "GAME_BEGIN") {
            current_round_hash = await window.appClient.startGame(gamecode);
            console.log("game started", current_round_hash);
            game_status = "MAKE_MOVE";
        } else if (action == "GAME_JOIN") {
            console.log("check if game has been started");
            let result = await window.appClient.currentRoundForGameCode(gamecode);
            if (!result) {
                alert("Still waiting on other players");
            } else {
                // get current round hash
                console.log("game joined", result);
                current_round_hash = result;
                game_status = "MAKE_MOVE";
            }
        }
    }

    export let current_round_hash;

    let rounds = [];
    let _max_round_counter = 0;

    // let last_round_state="IN PROGRESS";

    async function asyncCallZomeToMakeMove(event) {
        _max_round_counter = _max_round_counter + 1;
        rounds = [...rounds, { num: _max_round_counter, hash: "round_hash" }];

        game_status = "WAIT_NEXT_ROUND";
        let resources = event.detail.resources;
        console.log("taking resources:", resources);
        // (amount, prev_round_hash)
        let result = await window.appClient.makeMove(resources, current_round_hash);
    }

    function roundComplete() {
        if (_max_round_counter == 2) {
            // MAX ROUNDS
            game_status = "WAIT_GAME_SCORE";
            return;
        }
        if (game_status == "GAME_OVER") {
            return;
        }
        game_status = "MAKE_MOVE";
    }

    function callZomeToGetResults() {
        return new Promise((resolve) => {
            setTimeout(() => {
                resolve("resolved");
            }, DELAY);
        });
    }

    async function getAsyncFinalResults() {
        let result = await callZomeToGetResults();
        let mock_results = {
            total_score: 100,
            stats: [
                { nickname: "tixel", score: 10 },
                { nickname: "f00bar42", score: 10 },
                { nickname: "bierlitzm", score: 10 },
            ],
        };
        game_score = { totalscore: 100 };
        player_stats = mock_results.stats;
        game_status = "GAME_OVER";
        result_status = "GAME_LOST";
    }

    let players = [];
    let player_stats = [];
    let game_score = {};

    let players_mock_repo = [
        { nickname: "tixel", id: "56c95c9a-e210-41ec-8fec-fb9683c8d76f" },
        { nickname: "f00bar42", id: "4652cd28-4fc2-4c77-9709-234ca8adab81" },
        { nickname: "harlan", id: "c6b4f8a6-224f-4a7e-9a87-63416e0cafaf" },
        { nickname: "robot5x", id: "68472d6d-39d2-44cc-8c8c-5d21c8d75ae5" },
        { nickname: "lchang", id: "81ded8af-dcf1-407c-922d-20b9d7e3a42e" },
        {
            nickname: "sidsthalekar",
            id: "a5f78d5d-899e-458d-8b0e-98781c83d7ac",
        },
        {
            nickname: "guillemcordoba",
            id: "7ca195ac-71f3-4283-baa7-912d8a75e163",
        },
        {
            nickname: "petersgrandadventure",
            id: "ceef318d-4437-4485-8281-c47a018bc238",
        },
        { nickname: "nphias", id: "51797e5b-7306-4d6c-861b-a1a425112c37" },
        { nickname: "alexoceann", id: "4a0656a7-223e-4260-a84b-2b975f77eb52" },
        { nickname: "qubeo", id: "68ccf4f9-2ba3-42a1-87ca-9ffe562bca62" },
    ];
</script>

<section>
    <aside>
        <h3>Your nickname</h3>
        <span id="nickname" style="font-size: 4rem !important;">{nickname}</span
        >
    </aside>
    <aside>
        <h3>Game code</h3>
        <p style="font-size: 4rem !important;" id="gamecode">{gamecode}</p>
    </aside>

    <aside>
        <h3>Players</h3>
        <p style="font-size: 4rem !important;" id="playercount">
            {players.length}
        </p>
    </aside>
</section>

<div class="playerlist">
    {#each players as player, i}
        <button
            >{player.nickname}<br />{bufferToBase64(player.player_id)}</button
        >
    {/each}
</div>

<div class="gamerounds">
    {#if game_status == "WAITING_PLAYERS"}
        <div class="columncentered">
            <p>
                Wait until all player joined the game.
                <br />
                <a href="#" on:click={refreshPlayerList}>Refresh</a>
                <br />
                {#if players.length != 0}
                    <button class="startgamebutton" on:click={play}
                        >Play!</button
                    >
                {/if}
            </p>
        </div>
    {:else}
        <div class="columncentered">
            <a href="#" on:click={refreshPlayerList}>Refresh player list</a>
        </div>
        <!-- TODO for each list of rounds played-->
        {#each rounds as round, i}
            <GameRound {round} on:roundComplete={roundComplete} />
        {/each}
        {#if game_status == "MAKE_MOVE"}
            <GameMove on:makeMove={asyncCallZomeToMakeMove} />
        {/if}
        {#if game_status == "WAIT_NEXT_ROUND"}
            <div style="text-align:center;">
                Wait until round is complete...
                <br />
                Click refresh
            </div>
        {/if}
        {#if game_status == "WAIT_GAME_SCORE"}
            <div class="columncentered">
                <p>
                    Calculating game scores...
                    <br />
                    <a href="#" on:click={getAsyncFinalResults}>Click refresh</a
                    >
                </p>
            </div>
        {/if}

        <!-- ONLY if game ended-->
        {#if game_status == "GAME_OVER"}
            {#if result_status == "GAME_LOST"}
                <div style="text-align:center;">
                    <h1>
                        We lost
                        <br />
                        What a tragedy...
                    </h1>
                </div>
            {/if}
            {#if result_status == "GAME_WON"}
                <div style="text-align:center;">
                    <h1>Yes we made it... together.</h1>
                </div>
            {/if}
            <GameResults {player_stats} {game_score} />
        {/if}
    {/if}
</div>

<style>
    .gamerounds {
        display: flex;
        flex-direction: column;
        justify-content: center;
        margin-top: 1rem;
    }

    .columncentered {
        display: flex;
        flex-direction: column;
        justify-content: center;
        text-align: center;
        margin-top: 1rem;
    }

    .startgamebutton {
        text-align: center;
        width: 30%;
    }

    .playerlistRefresh {
        display: flex;
        justify-content: center;
        margin-top: 1rem;
    }

    .playerlist {
        padding-left: 10%;
        padding-right: 10%;
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        margin: 1rem;
    }

    .playerlist button {
        background-color: white;
        color: #118bee;
    }

    .playerlist button + button {
        margin-left: 1rem;
    }
</style>
