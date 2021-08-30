<script>
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();

    export let round = {
        num: 0,
        hash: "123"
    };

    $: roundnum = round.num;

    let moves = [
        {
            nickname: "tixel",
            id: "56c95c9a-e210-41ec-8fec-fb9683c8d76f",
            resourcesTaken: "10",
        },
    ];

    let moves_complete = [
        {
            nickname: "tixel",
            id: "56c95c9a-e210-41ec-8fec-fb9683c8d76f",
            resourcesTaken: "10",
        },
        {
            nickname: "f00bar42",
            id: "4652cd28-4fc2-4c77-9709-234ca8adab81",
            resourcesTaken: "10",
        },
        {
            nickname: "harlan",
            id: "c6b4f8a6-224f-4a7e-9a87-63416e0cafaf",
            resourcesTaken: "10",
        },
        {
            nickname: "robot5x",
            id: "68472d6d-39d2-44cc-8c8c-5d21c8d75ae5",
            resourcesTaken: "10",
        },
        {
            nickname: "lchang",
            id: "81ded8af-dcf1-407c-922d-20b9d7e3a42e",
            resourcesTaken: "10",
        },
    ];


    let gameRoundState = "IN PROGRESS"; // COMPLETE

    function refreshGameRound() {
        if (gameRoundState == "COMPLETE"){
            return;
        }
        if (moves.length < 2) {
            console.log("---------------> moves length {}", moves.length);
            moves = moves_complete;
            // dispatch("roundUpdated");
        // } else {
            // round complete
            gameRoundState = "COMPLETE";
            dispatch("roundComplete");
        }
    }
</script>

<section>
    <aside class="gameround">
        <h2>Round {roundnum} - {gameRoundState}</h2>
        <ul>
            {#each moves as move}
                <li>
                    {move.nickname} takes {move.resourcesTaken} resources<br
                    /><i style="color:silver">{move.id}</i>
                </li>
            {/each}
        </ul>
        {#if gameRoundState == "IN PROGRESS"}
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
