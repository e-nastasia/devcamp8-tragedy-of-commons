<script>
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();

    export let round = 1;
    $: roundnum = round + 1;

    let gameRoundState = "IN_PROGRESS"; // COMPLETE

    function refreshGameRound() {
        if (true) {
            // round complete
            gameRoundState = "COMPLETE";
            dispatch("roundComplete", {
                round: "1",
            });
        }
    }
</script>

<section>
    <aside class="gameround">
        <h2>Round {roundnum}</h2>
        <ul>
            <li>Tixel (3216549879654321654)</li>
            <li>E-nastasia (3216549879654321654)</li>
        </ul>
        {#if gameRoundState == "IN_PROGRESS"}
            <button on:click={refreshGameRound}>refresh</button>
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
