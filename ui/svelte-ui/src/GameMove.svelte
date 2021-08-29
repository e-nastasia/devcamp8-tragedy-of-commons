<script>
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    function makeMove(event){
        event.preventDefault();
        dispatch('message', {
			resources: '10'
		});
    }
</script>

<section>
    <form class="gamemove">
        <h1>Make your move</h1>
        <label for="input_join_nick">Total resources: 100</label>
        <p>Take all the resources you need</p>

        <label for="input_take_resources">Resources:</label>
        <input type="text" id="input_take_resources" size="20" placeholder="" />
        <button type="submit" on:click="{makeMove}">Submit</button>
    </form>
</section>

<style>
    .gamemove {
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
</style>
