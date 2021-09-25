<script>
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    export let total_resource = 100;

    function makeMove(event) {
        event.preventDefault();
        let input = document.getElementById("input_take_resources").value;
        dispatch("makeMove", {
            resources: input,
        });
    }
</script>

<section>
    <form class="gamemove">
        <h1>Make your move</h1>
        <label for="input_join_nick">Total resources: {total_resource}</label>
        <p>Take all the resources you need</p>

        <label for="input_take_resources">Resources:</label>
        <input
            type="number"
            id="input_take_resources"
            size="20"
            placeholder=""
        />
        <button id="make_move_btn" type="submit" on:click={makeMove}
            >Submit</button
        >
    </form>
</section>

<style>
    .gamemove {
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
</style>
