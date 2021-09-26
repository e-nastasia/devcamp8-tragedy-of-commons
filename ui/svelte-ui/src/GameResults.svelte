<script>
import { bufferToBase64, shorten } from "./utils";


    export let resources_left = 100;
    export let player_stats = [
        [ "mslfmsljfmlkds", 10 ],
        [ "xcvxcvj", 10 ],
        [ "zerzer", 10 ],
    ];
    export let players = [];

    function getPlayerName(id){
        for (let i = 0; i < players.length; i++) {
            const player = players[i];
            if (bufferToBase64(player.player_id) === id){
                return player.nickname;
            }
        }
    }
    
</script>

<section id="game_results_section">
    <aside class="gameresults">
        <h2>Game scores</h2>
        <p>Total score: {resources_left}</p>
        <ul>
            {#each player_stats as player}
                <li>
                    <strong>{getPlayerName(player[0])}</strong> took {player[1]} resources in total
                    <br><i class="playerId">{shorten(player[0])}</i> 
                </li>
            {/each}
        </ul>
    </aside>
</section>

<style>
    .playerId{
        color: silver;
    }
    .gameresults {
        width: 70%;
        background-color: lavender;
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
</style>
