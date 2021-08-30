<script>
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();
    export let nickname = "---";
    export let gamecode = "------";

    function startNewGame(event) {
        nickname = document.getElementById("input_start_nick").value;
        event.preventDefault();
        dispatch("startNewGame", {nickname});
    };
    function joinGame(event) {
        event.preventDefault();
        nickname = document.getElementById("input_join_nick").value;
        gamecode = document.getElementById("input_join_game_code").value;
        dispatch("joinGame", {nickname:nickname, gamecode:gamecode});
    }

</script>

<!-- <h1>Tragedy of the Commons</h1> -->
<div class="container">
    <form action="game.html" style="margin-right: 50px;">
        <h1>New game</h1>
        <p>Play a new game with others</p>
        <label for="input_start_nick">Nickname:</label>
        <input
            type="text"
            id="input_start_nick"
            name="nickname"
            size="20"
            placeholder=""
        />
        <input type="hidden" id="input_start_gamecode" name="gamecode" />
        <input type="hidden" name="action" value="start" />
        <button type="submit" on:click="{startNewGame}">Start</button>
    </form>
    <form action="game.html">
        <h1>Join game</h1>
        <p>Join a game with a game code</p>
        <label for="input_join_nick">Nickname:</label>
        <input type="text" id="input_join_nick" size="20" placeholder="" />
        <label for="input_join_game_code">Game code:</label>
        <input type="text" id="input_join_game_code" size="20" placeholder="" />
        <input type="hidden" id="input_join_gamecode" name="gamecode" />
        <input type="hidden" name="action" value="join" />
        <button type="submit" on:click="{joinGame}">Join</button>
    </form>
</div>

<style>
    h1 {
        color: #ff3e00;
        text-transform: uppercase;
        font-size: 4em;
        font-weight: 100;
    }
    .container {
        display: flex;
        justify-content: center;
        /* by default: align-items: strech -> child elements' heights = the container's height */
    }
    button {
        padding-left: 1rem;
        padding-right: 1rem;
    }
</style>
