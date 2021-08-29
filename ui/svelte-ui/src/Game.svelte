<script>
    import { each } from "svelte/internal";

    import GameMove from "./GameMove.svelte";
    import GameResults from "./GameResults.svelte";
    import GameRound from "./GameRound.svelte";

    let nickname = "Tixel";
    let gamecode = "3KL54M";
    let playercount = 3;

    let game_status = "WAITING_PLAYERS"; // "MAKE_MOVE" "WAIT_NEXT_ROUND" "GAME_LOST" "GAME_FINISHED"
    function refreshPlayerList() {
        alert("refreshing");
    }
    function beginGame() {
        game_status = "MAKE_MOVE";
    }
    let rounds = [];
    let _round_counter = 1;

    function makeMove() {
        if (_round_counter == 2) {
            game_status = "GAME_FINISHED";
        } else {
            _round_counter = _round_counter + 1;
            rounds = [...rounds, { _round_counter }];
            game_status = "WAIT_NEXT_ROUND";
        }
    }

    function roundComplete() {
        game_status = "MAKE_MOVE";
    }
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
            {playercount}
        </p>
    </aside>
</section>

<div class="playerlist">
    <button>E-nastasia<br />(3216549879654321654)</button>
    <button>Tixel<br />(3216549879654321654)</button>
    <button>Bierlingm<br />(3216549879654321654)</button>
    <button>E-nastasia<br />(3216549879654321654)</button>
    <button>Tixel<br />(3216549879654321654)</button>
    <button>Bierlingm<br />(3216549879654321654)</button>
    <button>E-nastasia<br />(3216549879654321654)</button>
    <button>Tixel<br />(3216549879654321654)</button>
    <button>Bierlingm<br />(3216549879654321654)</button>
</div>

<div class="gamerounds">
    {#if game_status == "WAITING_PLAYERS"}
        <div class="startgame">
            <p>
                Wait until all player joined the game.
                <br />
                <a href="#" on:click={refreshPlayerList}>Refresh</a>
                <br />
                <button class="startgamebutton" on:click={beginGame}
                    >Play!</button
                >
            </p>
        </div>
    {:else}
        <!-- TODO for each list of rounds played-->
        {#each rounds as round, i}
            <GameRound round={i} on:roundComplete={roundComplete} />
        {/each}
        {#if game_status == "MAKE_MOVE"}
            <GameMove on:message={makeMove} />
        {/if}
        {#if game_status == "WAIT_NEXT_ROUND"}
            <div style="text-align:center;">
                Wait until round is complete...
                <br />
                Click refresh
            </div>
        {/if}
        <!-- ONLY if game ended-->
        {#if game_status == "GAME_LOST" || game_status == "GAME_FINISHED"}
            {#if game_status == "GAME_LOST"}
            <div style="text-align:center;"
                We lost<br> What a tragedy...
            </div>
            {/if}
            {#if game_status == "GAME_FINISHED"}
            <div style="text-align:center;">
                Yes we made it... together.
            </div>
            {/if}
            <GameResults />
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

    .startgame {
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
