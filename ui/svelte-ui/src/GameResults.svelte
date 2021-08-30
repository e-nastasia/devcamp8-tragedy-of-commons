<script>
    export let game_scores = { totalscore: 100 };
    export let player_stats = [
        { nickname: "tixel", score: 10 },
        { nickname: "f00bar42", score: 10 },
        { nickname: "bierlitzm", score: 10 },
    ];
</script>

<section>
    <aside class="gameresults">
        <h2>Game scores</h2>
        <p>Total score: {game_scores.totalscore}</p>
        <ul>
            {#each player_stats as player}
                <li>
                    {player.nickname} took {player.score} resources in total
                </li>
            {/each}
        </ul>
    </aside>
</section>

<style>
    .gameresults {
        width: 70%;
        background-color: lavender;
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
</style>
