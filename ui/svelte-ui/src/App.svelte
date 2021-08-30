<script>
	import NavBar from "./NavBar.svelte";
	import StartMenu from "./StartMenu.svelte";
	import Game from "./Game.svelte";

	let status = "START"; // "GAME_BEGIN"  "GAME_JOIN" "LOADING"
	let nickname = "---";
	let gamecode = "------";

	function startNewGame(event) {
		if (status == "LOADING") {
			// prevent multiple clicks
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = generateGameCode();

		asyncCallZomeToStartNewGame();
	}

	function callZomeToStartNewGame() {
		return new Promise((resolve) => {
			setTimeout(() => {
				resolve("resolved");
			}, 1000);
		});
	}

	async function asyncCallZomeToStartNewGame() {
		// call holochain conductor
		// wait for response
		// move to other screen
		const result = await callZomeToStartNewGame();
		status = "GAME_BEGIN";
	}

	function joinGame(event) {
		if (status == "LOADING") {
			return;
		}
		status = "LOADING";
		nickname = event.detail.nickname;
		gamecode = event.detail.gamecode;
		// call holochain conductor
		// wait for response
		// move to other screen
		asyncCallZomeToJoinGame(gamecode)
	}

	function callZomeToJoinGame(gamecode) {
		return new Promise((resolve) => {
			setTimeout(() => {
				resolve("resolved");
			}, 1000);
		});
	}

	async function asyncCallZomeToJoinGame() {
		// call holochain conductor
		// wait for response
		// move to other screen
		const result = await callZomeToJoinGame();
		status = "GAME_JOIN";
	}

	function generateGameCode() {
		return Math.random().toString(36).substr(2, 6).toUpperCase();
	}
</script>

<NavBar />

{#if status == "START"}
	<StartMenu on:startNewGame={startNewGame} on:joinGame={joinGame} />
{:else if status == "LOADING"}
	<div style="flex-flow:column; text-align: center;">
		<h1>LOADING...</h1>
		<p>Connecting to conductor...</p>
		<p>Calling zome...</p>
	</div>
{:else}
	<Game action={status} {nickname} {gamecode} />
{/if}

<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
</style>
