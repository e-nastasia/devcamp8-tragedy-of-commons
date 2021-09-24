<script>
    import { createEventDispatcher } from "svelte";
import { shortenBase64 } from "./utils";
    const dispatch = createEventDispatcher();

    export let round = {
        round_num: 0,
        resources_left: 100,
        current_round_entry_hash: "slfsd",
        game_session_hash: "smdlfk",
        next_action: "TODO",
        moves: [
        {
            nickname: "tixel",
            id: "56c95c9a-e210-41ec-8fec-fb9683c8d76f",
            resourcesTaken: "10",
        }],
    };
    
    // moves
    export let moves = [];


    $: if (round.fake === false){
        gameRoundState = "COMPLETE";
    } else {
        gameRoundState = "IN PROGRESS";
    } 
    
    let gameRoundState = "NEW"; // COMPLETE

    function refreshGameRound() {
        if (gameRoundState == "COMPLETE"){
            return;
        }
        dispatch("updateRound");
    }
    function resourcesToString(amount){
        if (parseInt(amount) === 1) {
            return amount + " resource";
        } else {
            return amount + " resources";
        }
    }
</script>

<section>
    <aside class="gameround">
        <h2>Round {round.round_num} - {gameRoundState}<sup style="color:silver;background-color:white;">{shortenBase64(round.current_round_entry_hash)}</sup></h2>
        <ul>
            {#each moves as move}
            <li>
                {move.nickname} takes {resourcesToString(move.resourcesTaken)} <i style="color:silver;">{shortenBase64(move.id)}</i>
            </li>
            {/each}
        </ul>
        {#if gameRoundState == "IN PROGRESS"}
        <button id="refresh_round_btn" on:click={refreshGameRound}>refresh</button>
        {:else}
        <p>
            <strong>total resources: {round.resources_left}</strong>
        </p>
        {/if}
    </aside>
</section>

<style>
    .gameround {
        width: 70%;
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
    button {
        float: right;
        background: none !important;
        border: none;
        padding: 0 !important;
        /*optional*/
        font-family: arial, sans-serif;
        /*input has OS specific font-family*/
        color: #118bee;
        text-decoration: underline;
        cursor: pointer;
    }
</style>
